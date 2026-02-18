use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use daglang_derive::{derive_artifacts, DerivedArtifacts};
use daglang_emit::rust_exec_runtime::emit_exec_runtime;
use daglang_emit::{
    emit_c_bundle, emit_go_bundle, emit_mips_bundle, emit_rust_bundle, EmissionBundle,
    EmissionSummary,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompileOptions {
    pub emit_collection_nodes: bool,
    pub target: CodegenTarget,
    pub layer: CodegenLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodegenTarget {
    #[default]
    Rust,
    Go,
    C,
    Mips,
}

impl std::fmt::Display for CodegenTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Rust => "rust",
            Self::Go => "go",
            Self::C => "c",
            Self::Mips => "mips",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodegenLayer {
    /// Layer 1: Rust fast-path using gunbc-exec runtime.
    ExecRuntime,
    /// Layer 2: native codegen path through daglang-emit backend.
    #[default]
    Native,
}

impl std::fmt::Display for CodegenLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::ExecRuntime => "1",
            Self::Native => "2",
        };
        f.write_str(value)
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
    validate_module_path_consistency(
        &module_graph,
        &context.roots,
        context.target_file.as_deref(),
    )?;
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
    } else if options.emit_collection_nodes {
        lower_typed_project_with_collection_nodes(&typed)
    } else {
        lower_typed_project(&typed)
    }
    .map_err(|error| format!("lower error: {error}"))?;
    let derived = derive_artifacts(&lowered).map_err(|error| format!("derive error: {error}"))?;
    let emitted = emit_with_options(&lowered, &derived, options)
        .map_err(|error| format!("emit error: {error}"))?;

    Ok(CompileOutput {
        lowered_dag: lowered,
        derived,
        emitted,
    })
}

pub fn check_from_context(context: &DriverContext) -> Result<CheckOutput, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
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

fn emit_with_options(
    dag: &Dag<LoweredOp>,
    derived: &DerivedArtifacts,
    options: CompileOptions,
) -> Result<EmissionBundle, CompileError> {
    match (options.target, options.layer) {
        (CodegenTarget::Rust, CodegenLayer::Native) => {
            emit_rust_bundle(dag, derived).map_err(|error| {
                CompileError::from(format!("rust emit backend failed: {error}"))
            })
        }
        (CodegenTarget::Go, CodegenLayer::Native) => emit_go_bundle(dag, derived)
            .map_err(|error| CompileError::from(format!("go emit backend failed: {error}"))),
        (CodegenTarget::C, CodegenLayer::Native) => emit_c_bundle(dag, derived)
            .map_err(|error| CompileError::from(format!("c emit backend failed: {error}"))),
        (CodegenTarget::Mips, CodegenLayer::Native) => emit_mips_bundle(dag, derived)
            .map_err(|error| CompileError::from(format!("mips emit backend failed: {error}"))),
        (CodegenTarget::Rust, CodegenLayer::ExecRuntime) => {
            let module_name = derived
                .tool_metadata
                .modules
                .first()
                .map(|module| module.module.as_str())
                .unwrap_or("daglang.generated");
            let files = emit_exec_runtime(dag, module_name).map_err(|error| {
                CompileError::from(format!("rust exec-runtime emit failed: {error}"))
            })?;
            let callable_count = dag.nodes.len();
            let pipeline_count = dag
                .nodes
                .iter()
                .filter(|node| {
                    matches!(
                        &node.body,
                        gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline { .. })
                    )
                })
                .count();
            Ok(EmissionBundle {
                backend: "rust-exec-runtime".to_string(),
                files,
                summary: EmissionSummary {
                    module_count: derived.tool_metadata.modules.len(),
                    callable_count,
                    pipeline_count,
                },
            })
        }
        (target, CodegenLayer::ExecRuntime) => Err(CompileError::from(format!(
            "unsupported compile target/layer combination: --target {target} --layer 1; layer 1 currently supports only --target rust"
        ))),
    }
}

fn discover_module_graph_for_context(context: &DriverContext) -> Result<ModuleGraph, CompileError> {
    if let Some(target_file) = &context.target_file {
        return discover_target_module_graph_for_context(context, target_file);
    }

    ModuleGraph::discover_strict(&context.roots).map_err(format_resolve_error)
}

fn discover_target_module_graph_for_context(
    context: &DriverContext,
    target_file: &Path,
) -> Result<ModuleGraph, CompileError> {
    let canonical_roots = daglang_resolve::canonicalize_roots(&context.roots);
    let mut modules: Vec<ResolvedModule> = Vec::new();
    let mut imports_by_index: Vec<Vec<Vec<String>>> = Vec::new();
    let mut module_index_by_path: HashMap<PathBuf, usize> = HashMap::new();
    let mut module_index_by_decl: HashMap<Vec<String>, usize> = HashMap::new();

    let Some((target_index, _)) = add_target_module_if_applicable(
        target_file,
        None,
        &context.roots,
        &canonical_roots,
        &mut modules,
        &mut imports_by_index,
        &mut module_index_by_path,
        &mut module_index_by_decl,
    )?
    else {
        return Err(CompileError::from(
            "target file module path did not match expected import path",
        ));
    };

    let mut queue = VecDeque::from([target_index]);
    let mut visited = HashSet::new();
    while let Some(module_index) = queue.pop_front() {
        if !visited.insert(module_index) {
            continue;
        }
        let imports = imports_by_index
            .get(module_index)
            .cloned()
            .unwrap_or_default();
        let mut dependencies = Vec::new();
        for import in imports {
            if let Some(dep_index) = module_index_by_decl.get(&import).copied() {
                dependencies.push(dep_index);
                continue;
            }
            let Some(import_file) = resolve_import_file_path(&context.roots, &import) else {
                continue;
            };
            let Some((dep_index, is_new)) = add_target_module_if_applicable(
                &import_file,
                Some(&import),
                &context.roots,
                &canonical_roots,
                &mut modules,
                &mut imports_by_index,
                &mut module_index_by_path,
                &mut module_index_by_decl,
            )?
            else {
                continue;
            };
            dependencies.push(dep_index);
            if is_new {
                queue.push_back(dep_index);
            }
        }
        dependencies.sort_unstable();
        dependencies.dedup();
        if let Some(module) = modules.get_mut(module_index) {
            module.dependencies = dependencies;
        }
    }

    Ok(ModuleGraph { modules })
}

#[allow(clippy::too_many_arguments)]
fn add_target_module_if_applicable(
    path: &Path,
    expected_module_path: Option<&[String]>,
    roots: &[PathBuf],
    canonical_roots: &[PathBuf],
    modules: &mut Vec<ResolvedModule>,
    imports_by_index: &mut Vec<Vec<Vec<String>>>,
    module_index_by_path: &mut HashMap<PathBuf, usize>,
    module_index_by_decl: &mut HashMap<Vec<String>, usize>,
) -> Result<Option<(usize, bool)>, CompileError> {
    let canonical_path = {
        #[allow(clippy::disallowed_methods)]
        std::fs::canonicalize(path).ok()
    };
    if let Some(canonical_path) = canonical_path.as_ref() {
        if let Some(existing) = module_index_by_path.get(canonical_path).copied() {
            return Ok(Some((existing, false)));
        }
    }

    let (mut module, imports) = parse_target_module_file(path, roots, canonical_roots)?;
    if let Some(expected) = expected_module_path {
        if module.module_path.as_slice() != expected {
            return Ok(None);
        }
    }

    let canonical_path = match canonical_path {
        Some(path) => path,
        None =>
        {
            #[allow(clippy::disallowed_methods)]
            std::fs::canonicalize(path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?
        }
    };
    module.path = canonical_path.clone();

    if let Some(existing) = module_index_by_decl.get(&module.module_path).copied() {
        if modules
            .get(existing)
            .is_some_and(|existing_module| existing_module.path != canonical_path)
        {
            return Err(format_resolve_error(ResolveError::DuplicateModule(
                module.module_path.clone(),
            )));
        }
        module_index_by_path.insert(canonical_path, existing);
        return Ok(Some((existing, false)));
    }

    let index = modules.len();
    module_index_by_path.insert(canonical_path, index);
    module_index_by_decl.insert(module.module_path.clone(), index);
    imports_by_index.push(imports);
    modules.push(module);
    Ok(Some((index, true)))
}

fn parse_target_module_file(
    path: &Path,
    roots: &[PathBuf],
    canonical_roots: &[PathBuf],
) -> Result<(ResolvedModule, Vec<Vec<String>>), CompileError> {
    if path.is_dir() {
        return Err(format!(
            "failed to read {}: target is a directory; `.dag` paths are treated as single-file targets. Use `daglang check <dir>` or `daglang modules <dir>`, or pass the directory path without the `.dag` suffix.",
            path.display()
        )
        .into());
    }
    let source = {
        #[allow(clippy::disallowed_methods)]
        std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?
    };
    let ast = parser::parse_with_file_diagnostics(path, &source).map_err(|diagnostics| {
        format_resolve_error(ResolveError::ParseErrors(vec![(
            path.to_path_buf(),
            diagnostics,
        )]))
    })?;
    let module_path = ast
        .module_path
        .as_ref()
        .map(|module| module.node.segments.clone())
        .unwrap_or_else(|| daglang_resolve::path_to_module_path(path, roots, canonical_roots));
    let imports = ast
        .imports
        .iter()
        .map(|import| import.node.path.segments.clone())
        .collect::<Vec<_>>();
    Ok((
        ResolvedModule {
            path: path.to_path_buf(),
            ast,
            module_path,
            dependencies: Vec::new(),
        },
        imports,
    ))
}

fn resolve_import_file_path(roots: &[PathBuf], import_path: &[String]) -> Option<PathBuf> {
    let mut relative = PathBuf::new();
    for segment in import_path {
        relative.push(segment);
    }
    relative.set_extension("dag");
    roots
        .iter()
        .map(|root| root.join(&relative))
        .find(|candidate| candidate.is_file())
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
            CompileError::from(format!(
                "target file `{}` was not found in discovered module graph",
                target_file.display()
            ))
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
    target_file: Option<&Path>,
) -> Result<(), CompileError> {
    let mut root_prefixes = roots.to_vec();
    for canonical_root in daglang_resolve::canonicalize_roots(roots) {
        if !root_prefixes.contains(&canonical_root) {
            root_prefixes.push(canonical_root);
        }
    }
    let canonical_target = target_file.and_then(|target| {
        #[allow(clippy::disallowed_methods)]
        std::fs::canonicalize(target).ok()
    });
    let mismatches = graph
        .modules
        .iter()
        .filter_map(|module| {
            if target_file.is_some_and(|target| module.path == target)
                || canonical_target
                    .as_ref()
                    .is_some_and(|canonical| module.path == *canonical)
            {
                return None;
            }
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
                ..CompileOptions::default()
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

    #[test]
    fn compile_with_exec_runtime_layer_emits_exec_runtime_bundle() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/makegen.dag");

        let context = DriverContext {
            roots: vec![root],
            target_file: Some(file),
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                layer: CodegenLayer::ExecRuntime,
                ..CompileOptions::default()
            },
        )
        .expect("compile should succeed with rust exec-runtime layer");

        assert_eq!(output.emitted.backend, "rust-exec-runtime");
        assert!(output
            .emitted
            .files
            .iter()
            .any(|file| file.path == "src/main.rs"));
        assert!(output
            .emitted
            .files
            .iter()
            .any(|file| file.path == "Cargo.toml"));
    }

    #[test]
    fn compile_with_non_rust_exec_runtime_layer_reports_error() {
        let root = unique_temp_dir("compile_unsupported_target");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(
            &file,
            r#"module sample
fn run() -> Bool {
  return true
}
"#,
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let error = compile_from_context_with_options(
            &context,
            CompileOptions {
                target: CodegenTarget::Go,
                layer: CodegenLayer::ExecRuntime,
                ..CompileOptions::default()
            },
        )
        .expect_err("compile should fail for unsupported target");
        assert!(
            error
                .as_str()
                .contains("layer 1 currently supports only --target rust"),
            "expected unsupported target error, got: {error}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_with_go_native_layer_emits_go_bundle() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/makegen.dag");
        let context = DriverContext {
            roots: vec![root],
            target_file: Some(file),
        };

        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                target: CodegenTarget::Go,
                ..CompileOptions::default()
            },
        )
        .expect("compile should succeed for go native layer");
        assert_eq!(output.emitted.backend, "go");
        assert!(output
            .emitted
            .files
            .iter()
            .any(|file| file.path == "target/generated/go/main.go"));
    }

    /// D1.7 — Structural verification that exec-runtime codegen for the real
    /// makegen.dag produces correct code.
    ///
    /// This test compiles the actual `dsl/tools/makegen.dag` through the full
    /// pipeline and verifies the generated main.rs contains:
    /// - All expected handler kinds (content upsert chain + entrypoint + render)
    /// - Correct DAG topology (matching the lowered DAG)
    /// - Correct entrypoint argument parsing
    /// - Valid Cargo.toml with required dependencies
    #[test]
    fn makegen_exec_runtime_e2e_structural_verification() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/makegen.dag");

        let context = DriverContext {
            roots: vec![root],
            target_file: Some(file),
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                layer: CodegenLayer::ExecRuntime,
                ..CompileOptions::default()
            },
        )
        .expect("compile should succeed for makegen exec-runtime");

        let main_rs = output
            .emitted
            .files
            .iter()
            .find(|f| f.path == "src/main.rs")
            .expect("should emit src/main.rs")
            .content
            .as_str();
        let cargo_toml = output
            .emitted
            .files
            .iter()
            .find(|f| f.path == "Cargo.toml")
            .expect("should emit Cargo.toml")
            .content
            .as_str();

        // ---- Handler kinds ----
        // The content upsert pattern should produce all these handler kinds:
        assert!(
            main_rs.contains("LoadRegistry"),
            "missing LoadRegistry handler"
        );
        assert!(
            main_rs.contains("RenderMakefile"),
            "missing RenderMakefile handler"
        );
        assert!(main_rs.contains("Entrypoint"), "missing Entrypoint handler");
        assert!(
            main_rs.contains("PrepareReadContent"),
            "missing PrepareReadContent handler"
        );
        assert!(
            main_rs.contains("ExecuteReadContent"),
            "missing ExecuteReadContent handler"
        );
        assert!(
            main_rs.contains("CompareContent"),
            "missing CompareContent handler"
        );
        assert!(
            main_rs.contains("PrepareWriteContent"),
            "missing PrepareWriteContent handler"
        );
        assert!(
            main_rs.contains("ExecuteTransport"),
            "missing ExecuteTransport handler"
        );

        // ---- DAG topology ----
        // The number of add_node calls should match the lowered DAG node count.
        let expected_nodes = output.lowered_dag.nodes.len();
        let actual_nodes = main_rs.matches("dag.add_node").count();
        assert_eq!(
            actual_nodes, expected_nodes,
            "generated DAG should have {expected_nodes} nodes, got {actual_nodes}"
        );

        // The number of add_edge calls should match the lowered DAG edge count.
        let expected_edges = output.lowered_dag.edges.len();
        let actual_edges = main_rs.matches("dag.add_edge").count();
        assert_eq!(
            actual_edges, expected_edges,
            "generated DAG should have {expected_edges} edges, got {actual_edges}"
        );

        // Every node ID from the lowered DAG should appear in the generated code.
        for node in &output.lowered_dag.nodes {
            assert!(
                main_rs.contains(&node.id.0),
                "generated code should reference node `{}`",
                node.id.0
            );
        }

        // ---- Entrypoint parsing ----
        // makegen has an entrypoint port for the output path — the generated
        // main should parse it from CLI args.
        assert!(
            main_rs.contains("input_mocks"),
            "generated main should set up input mocks for entrypoints"
        );

        // ---- Executable impl structure ----
        assert!(
            main_rs.contains("impl Executable for Op"),
            "should contain Executable impl"
        );
        assert!(
            main_rs.contains("fn execute("),
            "should contain execute method"
        );
        assert!(
            main_rs.contains("fn build_dag()"),
            "should contain build_dag function"
        );
        assert!(
            main_rs.contains("fn main()"),
            "should contain main function"
        );
        assert!(
            main_rs.contains("execute_with_mode_and_inputs"),
            "main should call the executor"
        );

        // ---- Handler body correctness ----
        // The render_makefile handler should produce "Generated by daglang" header.
        assert!(
            main_rs.contains("Generated by daglang"),
            "render_makefile handler should contain Makefile header text"
        );
        // The compare handler should check freshness.
        assert!(
            main_rs.contains("fresh"),
            "compare handler should compute freshness"
        );
        // The execute_transport handler should respect skip flag.
        assert!(
            main_rs.contains("Value::Skipped"),
            "execute_transport handler should handle skip"
        );

        // ---- Cargo.toml ----
        assert!(
            cargo_toml.contains("gunbc-ir"),
            "Cargo.toml should depend on gunbc-ir"
        );
        assert!(
            cargo_toml.contains("gunbc-exec"),
            "Cargo.toml should depend on gunbc-exec"
        );
        assert!(
            cargo_toml.contains("gunbc-lib-transport"),
            "Cargo.toml should depend on gunbc-lib-transport"
        );
        assert!(
            cargo_toml.contains(r#"name = "tools-makegen""#),
            "Cargo.toml should have sanitized crate name"
        );
    }

    /// D1.8 — Structural verification that exec-runtime codegen for the real
    /// pragma.dag produces correct code with 3 parallel content upsert chains.
    #[test]
    fn pragma_exec_runtime_e2e_structural_verification() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/pragma.dag");

        let context = DriverContext {
            roots: vec![root],
            target_file: Some(file),
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                layer: CodegenLayer::ExecRuntime,
                ..CompileOptions::default()
            },
        )
        .expect("compile should succeed for pragma exec-runtime");

        let main_rs = output
            .emitted
            .files
            .iter()
            .find(|f| f.path == "src/main.rs")
            .expect("should emit src/main.rs")
            .content
            .as_str();
        let cargo_toml = output
            .emitted
            .files
            .iter()
            .find(|f| f.path == "Cargo.toml")
            .expect("should emit Cargo.toml")
            .content
            .as_str();

        // ---- Pragma-specific handler kinds ----
        assert!(
            main_rs.contains("RenderPragmaClippyToml"),
            "missing RenderPragmaClippyToml handler"
        );
        assert!(
            main_rs.contains("RenderPragmaAllowlist"),
            "missing RenderPragmaAllowlist handler"
        );
        assert!(
            main_rs.contains("RenderPragmaLintPolicy"),
            "missing RenderPragmaLintPolicy handler"
        );
        assert!(
            main_rs.contains("PragmaEntrypoint"),
            "missing PragmaEntrypoint handler"
        );

        // ---- Content upsert pattern handlers (shared) ----
        assert!(
            main_rs.contains("PrepareReadContent"),
            "missing PrepareReadContent handler"
        );
        assert!(
            main_rs.contains("ExecuteReadContent"),
            "missing ExecuteReadContent handler"
        );
        assert!(
            main_rs.contains("CompareContent"),
            "missing CompareContent handler"
        );
        assert!(
            main_rs.contains("PrepareWriteContent"),
            "missing PrepareWriteContent handler"
        );
        assert!(
            main_rs.contains("ExecuteTransport"),
            "missing ExecuteTransport handler"
        );

        // ---- Pragma helper infrastructure ----
        assert!(
            main_rs.contains("PragmaDirectiveRuntime"),
            "should emit PragmaDirectiveRuntime struct"
        );
        assert!(
            main_rs.contains("parse_pragma_directives"),
            "should emit pragma directive parsing helper"
        );

        // ---- DAG topology ----
        // Pragma has 3 parallel chains (clippy, allowlist, policy) each with
        // 5 content-upsert nodes, plus render nodes, fs_env, and entrypoint.
        let expected_nodes = output.lowered_dag.nodes.len();
        let actual_nodes = main_rs.matches("dag.add_node").count();
        assert_eq!(
            actual_nodes, expected_nodes,
            "generated DAG should have {expected_nodes} nodes, got {actual_nodes}"
        );

        let expected_edges = output.lowered_dag.edges.len();
        let actual_edges = main_rs.matches("dag.add_edge").count();
        assert_eq!(
            actual_edges, expected_edges,
            "generated DAG should have {expected_edges} edges, got {actual_edges}"
        );

        // Every node ID from the lowered DAG should appear in the generated code.
        for node in &output.lowered_dag.nodes {
            assert!(
                main_rs.contains(&node.id.0),
                "generated code should reference node `{}`",
                node.id.0
            );
        }

        // ---- Handler body correctness ----
        // Pragma render handlers should filter directives by scope.
        assert!(
            main_rs.contains("clippy"),
            "clippy render handler should filter by clippy scope"
        );
        assert!(
            main_rs.contains("disallowed_method"),
            "allowlist render handler should filter by disallowed_method key"
        );
        assert!(
            main_rs.contains("lint"),
            "lint policy render handler should filter by lint scope"
        );

        // ---- Cargo.toml ----
        assert!(
            cargo_toml.contains(r#"name = "tools-pragma""#),
            "Cargo.toml should have sanitized crate name"
        );
    }

}
