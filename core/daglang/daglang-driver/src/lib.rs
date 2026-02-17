use std::fmt::Write;
use std::path::PathBuf;

use daglang_derive::{derive_artifacts, DerivedArtifacts};
use daglang_emit::{emit_rust_bundle, EmissionBundle};
use daglang_lower::{lower_typed_project, LoweredOp};
use daglang_resolve::{ModuleGraph, ResolveError, ResolvedModule};
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

impl std::ops::Deref for CompileError {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.message.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOutput {
    pub parsed_files: usize,
}

pub fn compile_from_context(context: &DriverContext) -> Result<CompileOutput, CompileError> {
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

pub fn check_from_context(context: &DriverContext) -> Result<CheckOutput, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    if context.target_file.is_none() {
        validate_module_path_consistency(&module_graph, &context.roots)?;
    }
    let parsed_files = module_graph.modules.len();
    typecheck_module_graph_with_options(
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
    Ok(CheckOutput { parsed_files })
}

#[allow(clippy::disallowed_methods)]
fn discover_module_graph_for_context(context: &DriverContext) -> Result<ModuleGraph, CompileError> {
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
    let mismatches = graph
        .modules
        .iter()
        .filter_map(|module| {
            let declared = module.module_path.join(".");
            let relative = roots.iter().find_map(|root| {
                module
                    .path
                    .strip_prefix(root)
                    .ok()
                    .map(PathBuf::from)
            })?;
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
mod tests {
    use super::*;
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
        assert!(error.contains("module path mismatches"));
        assert!(error.contains("declared `mismatch.main`"));

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
}
