use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use daglang_derive::{derive_artifacts, DerivedArtifacts};
use daglang_emit::{emit_rust_bundle, EmissionBundle};
use daglang_lower::{
    lower_typed_project, lower_typed_project_for_modules,
    lower_typed_project_for_modules_with_collection_nodes,
    lower_typed_project_with_collection_nodes, LoweredOp,
};
use daglang_resolve::{ModuleGraph, ResolveError, ResolvedModule};
use daglang_syntax::ast::Item;
use daglang_syntax::diagnostic;
use daglang_syntax::parser;
use daglang_typecheck::{typecheck_module_graph_with_options, TypecheckOptions};
use gunbc_ir::Dag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverContext {
    pub roots: Vec<PathBuf>,
    pub target_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct CompileOutput {
    pub lowered_dag: Dag<LoweredOp>,
    pub derived: DerivedArtifacts,
    pub emitted: EmissionBundle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    message: String,
}

impl CompileError {
    pub fn as_str(&self) -> &str {
        self.message.as_str()
    }

    pub fn contains(&self, needle: &str) -> bool {
        self.message.contains(needle)
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for CompileError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for CompileError {
    fn from(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOutput {
    pub parsed_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompileOptions {
    pub emit_collection_nodes: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            emit_collection_nodes: false,
        }
    }
}

pub fn compile_from_context(context: &DriverContext) -> Result<CompileOutput, CompileError> {
    compile_from_context_with_options(context, CompileOptions::default())
}

pub fn compile_from_context_with_options(
    context: &DriverContext,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    let callable_scope = callable_scope_for_context(context, &module_graph)?;
    validate_module_path_consistency(&module_graph, &context.roots)?;
    let typed = typecheck_module_graph_with_options(
        module_graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .map_err(format_typecheck_errors)?;
    let lowered = if let Some(scope) = callable_scope.as_ref() {
        if options.emit_collection_nodes {
            lower_typed_project_for_modules_with_collection_nodes(&typed, scope)
        } else {
            lower_typed_project_for_modules(&typed, scope)
        }
    } else {
        if options.emit_collection_nodes {
            lower_typed_project_with_collection_nodes(&typed)
        } else {
            lower_typed_project(&typed)
        }
    }
    .map_err(|error| format!("lower error: {error}"))?;
    let derived = derive_artifacts(&lowered).map_err(|error| format!("derive error: {error}"))?;
    let emitted =
        emit_rust_bundle(&lowered, &derived).map_err(|error| format!("emit error: {error}"))?;

    Ok(CompileOutput {
        lowered_dag: lowered,
        derived,
        emitted,
    })
}

pub fn check_from_context(context: &DriverContext) -> Result<CheckOutput, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    validate_module_path_consistency(&module_graph, &context.roots)?;
    let parsed_files = module_graph.modules.len();
    if let Err(errors) = typecheck_module_graph_with_options(
        module_graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    ) {
        return Err(format_typecheck_errors(errors));
    }
    Ok(CheckOutput { parsed_files })
}

fn format_typecheck_errors<E: std::fmt::Display>(errors: Vec<E>) -> CompileError {
    let mut message = String::from("typecheck errors:\n");
    for error in errors {
        writeln!(message, "  {error}").ok();
    }
    message.into()
}

fn discover_module_graph_for_context(context: &DriverContext) -> Result<ModuleGraph, CompileError> {
    if let Some(target_file) = &context.target_file {
        let single_file_graph = discover_single_file_module_graph(target_file)?;
        let module_path = single_file_graph.modules[0].module_path.clone();
        let discovered =
            ModuleGraph::discover_strict(&context.roots).map_err(format_resolve_error)?;
        return prune_module_graph_to_target(discovered, &module_path).ok_or_else(|| {
            format!(
                "target module `{}` was not discovered under configured roots",
                module_path.join(".")
            )
            .into()
        });
    }

    ModuleGraph::discover_strict(&context.roots).map_err(format_resolve_error)
}

fn discover_single_file_module_graph(target_file: &Path) -> Result<ModuleGraph, CompileError> {
    let source = {
        #[allow(clippy::disallowed_methods)]
        std::fs::read_to_string(target_file)
            .map_err(|error| format!("failed to read {}: {error}", target_file.display()))?
    };
    let ast = parser::parse(&source).map_err(|errors| {
        let mut message = String::from("compile diagnostics:\n");
        for error in &errors {
            writeln!(
                message,
                "  {}",
                error.format_with_source(target_file, &source)
            )
            .ok();
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
    Ok(ModuleGraph {
        modules: vec![ResolvedModule {
            path: target_file.to_path_buf(),
            ast,
            module_path,
            dependencies: Vec::new(),
        }],
    })
}

fn callable_scope_for_context(
    context: &DriverContext,
    module_graph: &ModuleGraph,
) -> Result<Option<HashSet<String>>, CompileError> {
    let Some(target_file) = context.target_file.as_ref() else {
        return Ok(None);
    };
    let canonical_target = {
        #[allow(clippy::disallowed_methods)]
        std::fs::canonicalize(target_file).ok()
    };
    let target_index = module_graph
        .modules
        .iter()
        .position(|module| {
            module.path == *target_file
                || canonical_target
                    .as_ref()
                    .is_some_and(|canonical| module.path == *canonical)
        })
        .ok_or_else(|| {
            format!(
                "target file `{}` was not found in discovered module graph",
                target_file.display()
            )
            .into()
        })?;
    let Some(target_module) = module_graph.modules.get(target_index) else {
        return Err("internal error: target module index out of bounds".into());
    };
    let has_callable_items = module_has_callable_items(target_module);
    if !has_callable_items {
        return Ok(None);
    }
    let has_pipeline_items = target_module
        .ast
        .items
        .iter()
        .any(|item| matches!(item.node, Item::PipelineDef(_)));
    let mut scope = HashSet::new();
    if !has_pipeline_items {
        scope.insert(target_module.module_path.join("."));
        return Ok(Some(scope));
    }
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([target_index]);
    while let Some(module_index) = queue.pop_front() {
        if !visited.insert(module_index) {
            continue;
        }
        let Some(module) = module_graph.modules.get(module_index) else {
            continue;
        };
        scope.insert(module.module_path.join("."));
        for dependency in &module.dependencies {
            queue.push_back(*dependency);
        }
    }
    if scope.is_empty() {
        scope.insert(target_module.module_path.join("."));
    }
    Ok(Some(scope))
}

fn module_has_callable_items(module: &ResolvedModule) -> bool {
    module.ast.items.iter().any(|item| {
        matches!(
            item.node,
            Item::FnDef(_) | Item::FuncDef(_) | Item::PatternDef(_) | Item::PipelineDef(_)
        )
    })
}

fn prune_module_graph_to_target(
    graph: ModuleGraph,
    target_module_path: &[String],
) -> Option<ModuleGraph> {
    let target_module = target_module_path.join(".");
    let target_index = graph
        .modules
        .iter()
        .position(|module| module.module_path.join(".") == target_module)?;
    if target_index >= graph.modules.len() {
        return None;
    }
    let mut required = HashSet::new();
    let mut queue = VecDeque::from([target_index]);
    while let Some(module_index) = queue.pop_front() {
        if !required.insert(module_index) {
            continue;
        }
        let Some(module) = graph.modules.get(module_index) else {
            continue;
        };
        for dependency in &module.dependencies {
            queue.push_back(*dependency);
        }
    }
    let mut index_map = HashMap::new();
    for (old_index, _) in graph.modules.iter().enumerate() {
        if required.contains(&old_index) {
            let new_index = index_map.len();
            index_map.insert(old_index, new_index);
        }
    }
    let modules = graph
        .modules
        .into_iter()
        .enumerate()
        .filter_map(|(old_index, mut module)| {
            if !required.contains(&old_index) {
                return None;
            }
            module.dependencies = module
                .dependencies
                .iter()
                .filter_map(|dependency| index_map.get(dependency).copied())
                .collect::<Vec<_>>();
            Some(module)
        })
        .collect::<Vec<_>>();
    Some(ModuleGraph { modules })
}

fn format_resolve_error(error: ResolveError) -> CompileError {
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
            message.into()
        }
        other => format!("resolve error: {other}").into(),
    }
}

fn validate_module_path_consistency(
    graph: &ModuleGraph,
    roots: &[PathBuf],
) -> Result<(), CompileError> {
    let mut root_prefixes = roots.to_vec();
    for canonical_root in daglang_resolve::canonicalize_roots(roots) {
        if !root_prefixes.contains(&canonical_root) {
            root_prefixes.push(canonical_root);
        }
    }
    let mismatches = graph
        .modules
        .iter()
        .filter_map(|module| {
            let declared = module.module_path.join(".");
            let relative = root_prefixes
                .iter()
                .find_map(|root| module.path.strip_prefix(root).ok().map(PathBuf::from))?;
            let mut inferred_segments = Vec::new();
            for component in relative.components() {
                use std::path::Component;
                if let Component::Normal(part) = component {
                    inferred_segments.push(part.to_string_lossy().to_string());
                }
            }
            if let Some(last) = inferred_segments.last_mut() {
                if let Some(stripped) = last.strip_suffix(".dag") {
                    *last = stripped.to_string();
                }
            }
            if inferred_segments.join(".") == declared {
                None
            } else {
                Some(format!(
                    "{}: declared `{}` but filesystem implies `{}`",
                    module.path.display(),
                    declared,
                    inferred_segments.join("."),
                ))
            }
        })
        .collect::<Vec<_>>();

    if mismatches.is_empty() {
        Ok(())
    } else {
        let mut message = String::from("module path mismatches:\n");
        for mismatch in mismatches {
            writeln!(message, "  {mismatch}").ok();
        }
        Err(message.into())
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "daglang_driver_{label}_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ))
    }

    #[test]
    fn compile_directory_reports_module_path_mismatch() {
        let root = unique_temp_dir("module_mismatch");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        std::fs::write(
            root.join("main.dag"),
            "module mismatch.main\nfn run() -> Unit {}",
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let error = compile_from_context(&context).expect_err("compile should fail");
        let error_text = error.as_str();
        assert!(error_text.contains("module path mismatches"));
        assert!(error_text.contains("declared `mismatch.main`"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn check_single_file_valid_source_succeeds() {
        let root = unique_temp_dir("check_single_file");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(&file, "module sample\nfn run() -> Unit {}\n")
            .expect("failed to write valid source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = check_from_context(&context).expect("check should succeed");
        assert_eq!(output.parsed_files, 1);

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn check_single_file_includes_discovered_dependency_closure() {
        let root = unique_temp_dir("check_single_file_with_deps");
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/dep.dag"),
            "module sample.dep\ntype Thing = String\n",
        )
        .expect("failed to write dependency source");
        let file = root.join("sample/main.dag");
        std::fs::write(
            &file,
            "module sample.main\nimport sample.dep { Thing }\nfn run(v: Thing) -> Thing { v }\n",
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = check_from_context(&context).expect("check should succeed");
        assert_eq!(
            output.parsed_files, 2,
            "single-file check should include dependency closure when discovery succeeds"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn check_single_file_dependency_discovery_failure_reports_error() {
        let root = unique_temp_dir("check_single_file_fallback");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(
            &file,
            "module sample\nimport missing.dep\nfn run() -> Unit {}\n",
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let error =
            check_from_context(&context).expect_err("strict mode should fail unresolved import");
        assert!(
            error.as_str().contains("unresolved import"),
            "expected unresolved import error, got: {error}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn check_single_file_with_dependency_closure_does_not_relax_unresolved_imports() {
        let root = unique_temp_dir("check_single_file_strict_with_deps");
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/dep.dag"),
            "module sample.dep\ntype Thing = String\n",
        )
        .expect("failed to write dependency source");
        let file = root.join("sample/main.dag");
        std::fs::write(
            &file,
            "module sample.main\nimport sample.dep { Thing }\nfn run(v: Thing) -> Thing { unresolved_call(v) }\n",
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let error =
            check_from_context(&context).expect_err("strict dependency closure should typecheck");
        assert!(
            error.as_str().contains("unresolved call target"),
            "expected unresolved call target error, got: {error}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_pipeline_target_includes_callable_dependency_closure() {
        let root = unique_temp_dir("compile_pipeline_scope_with_deps");
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/helper.dag"),
            "module sample.helper\nfn dep_task() -> Bool { true }\n",
        )
        .expect("failed to write helper source");
        let file = root.join("sample/main.dag");
        std::fs::write(
            &file,
            "module sample.main\nimport sample.helper { dep_task }\npipeline run { stage only { dep = dep_task() } }\n",
        )
        .expect("failed to write pipeline source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = compile_from_context(&context).expect("compile should succeed");
        let node_ids = output
            .lowered_dag
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        assert!(
            node_ids.contains("sample.helper::dep_task"),
            "pipeline single-file compile should include callable dependencies"
        );
        assert!(node_ids.contains("sample.main::run"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_function_target_keeps_callable_scope_local() {
        let root = unique_temp_dir("compile_function_scope_local");
        std::fs::create_dir_all(root.join("sample")).expect("failed to create temp root");
        std::fs::write(
            root.join("sample/helper.dag"),
            "module sample.helper\nfn dep_task() -> Bool { true }\n",
        )
        .expect("failed to write helper source");
        let file = root.join("sample/main.dag");
        std::fs::write(
            &file,
            "module sample.main\nimport sample.helper { dep_task }\nfn run() -> Bool { dep_task() }\n",
        )
        .expect("failed to write function source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = compile_from_context(&context).expect("compile should succeed");
        let node_ids = output
            .lowered_dag
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        assert!(
            !node_ids.contains("sample.helper::dep_task"),
            "function single-file compile should keep callable scope local"
        );
        assert!(node_ids.contains("sample.main::run"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_with_collection_option_emits_collection_nodes() {
        let root = unique_temp_dir("compile_collection_nodes");
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

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                emit_collection_nodes: true,
            },
        )
        .expect("compile should succeed with collection nodes enabled");
        let node_ids = output
            .lowered_dag
            .nodes
            .iter()
            .map(|node| node.id.0.clone())
            .collect::<HashSet<_>>();
        assert!(node_ids.contains("sample::run::MapNode_0"));
        assert!(node_ids.contains("sample::run::JoinNode_1"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }
}
