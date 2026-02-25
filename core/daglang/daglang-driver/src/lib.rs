use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use daglang_derive::{derive_artifacts, DerivedArtifacts};
use daglang_emit::rust_exec_runtime::emit_exec_runtime_with_output_dir;
use daglang_emit::{
    emit_c_bundle, emit_go_bundle, emit_mips_bundle, emit_rust_bundle, EmissionBundle, EmittedFile,
    EmissionSummary,
};
pub use daglang_lower::InferredEntrypoint;
pub use daglang_lower::is_user_param_port;
use daglang_lower::{
    lower_typed_project_for_modules_with_entry,
    lower_typed_project_for_modules_with_entry_and_collection_nodes,
    lower_typed_project_with_profile, lower_typed_project_with_profile_and_collection_nodes,
    LoweredOp,
};
use daglang_resolve::{ModuleGraph, ResolveError, ResolvedModule};
use daglang_syntax::ast::{Expr, Item, Literal, PipelineDef, StageDef, Stmt, TypeBody};
use daglang_syntax::ast_utils::type_expr_to_string;
use daglang_syntax::diagnostic;
use daglang_syntax::parser;
use daglang_typecheck::{typecheck_module_graph_with_options, TypecheckOptions, TypedProject};
use gunbc_ir::{Dag, ProgramSymbolId, ReachableDag, TypeRegistry};
use serde::Serialize;
use sha2::{Digest, Sha256};

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
    /// Relative path of the deterministic emission manifest.
    pub emit_manifest_path: String,
    /// All output file paths this tool produces, auto-extracted from
    /// `content_upsert` literal paths and `@outputs` annotations.
    pub output_paths: Vec<String>,
    /// Pipeline-level `param` declarations extracted from the DSL source.
    /// Each entry includes the param name, type, and optional default value.
    pub pipeline_params: Vec<PipelineParam>,
    /// Entrypoints inferred from graph structure: `func` items with untapped inputs.
    pub inferred_entrypoints: Vec<InferredEntrypoint>,
    /// Type registry extracted from DSL-defined sum and product types.
    ///
    /// Contains coproduct/product registrations for all `type` definitions in
    /// the compiled modules. Merge into `TypeRegistry::with_core_types()` to
    /// make DSL-defined types visible to testgen.
    pub dsl_type_registry: TypeRegistry,
    /// Compile receipt with deterministic digests.
    ///
    /// `None` if receipt computation was not requested.
    pub receipt: Option<CompileReceipt>,
}

/// Deterministic compilation receipt.
///
/// Contains content-addressable digests for each stage of the compilation
/// pipeline. Two compilations of the same input must produce identical
/// receipts — this is the determinism contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompileReceipt {
    /// SHA-256 of sorted source file content hashes.
    pub source_digest: String,
    /// SHA-256 of the canonical IR JSON representation.
    pub program_ir_digest: String,
    /// SHA-256 of the sorted emit manifest JSON.
    pub emit_manifest_digest: String,
}

/// A pipeline-level parameter declaration from `param name: Type = default`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineParam {
    pub name: String,
    pub type_id: String,
    pub default_value: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompileOptions {
    pub emit_collection_nodes: bool,
    pub profile: Option<String>,
    pub target: CodegenTarget,
    pub layer: CodegenLayer,
    /// Optional output directory for emitted files.
    ///
    /// Used by emitters that need to derive relative paths in generated
    /// artifacts (for example Cargo.toml workspace path dependencies).
    pub output_dir: Option<PathBuf>,
    /// Pre-computed embedded data keyed by `"module::semantic_key"`.
    /// Go/C/MIPS backends embed these as string literals; Rust Layer 1
    /// writes them as additional files in the generated crate.
    pub embedded_data: std::collections::HashMap<String, daglang_emit::EmbeddedData>,
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
    compile_from_module_graph_with_options(context, module_graph, options)
}

/// Compile from a pre-built module graph, skipping discovery.
///
/// This is the shared compilation path used by both the direct context-based
/// flow and the pipeline-based flow (DL5). The pipeline handles discovery,
/// parsing, and module graph construction; this function handles validation,
/// typechecking, lowering, and emission.
pub fn compile_from_module_graph_with_options(
    context: &DriverContext,
    mut module_graph: ModuleGraph,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    include_profile_modules(&mut module_graph, &context.roots, options.profile.is_some())?;
    let callable_scope_result = callable_scope_for_context(context, &module_graph)?;
    let (callable_scope, entry_module_name) = match callable_scope_result {
        Some((scope, entry)) => (Some(scope), Some(entry)),
        None => (None, None),
    };
    // Save source file paths before typechecking consumes the module graph.
    let source_paths: Vec<PathBuf> = module_graph
        .modules
        .iter()
        .map(|m| m.path.clone())
        .collect();
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
    let extern_assets = collect_extern_assets(&typed);
    let lowered = if let Some(scope) = callable_scope.as_ref() {
        if options.emit_collection_nodes {
            lower_typed_project_for_modules_with_entry_and_collection_nodes(
                &typed,
                scope,
                options.profile.as_deref(),
                entry_module_name.as_deref(),
            )
        } else {
            lower_typed_project_for_modules_with_entry(
                &typed,
                scope,
                options.profile.as_deref(),
                entry_module_name.as_deref(),
            )
        }
    } else if options.emit_collection_nodes {
        lower_typed_project_with_profile_and_collection_nodes(&typed, options.profile.as_deref())
    } else {
        lower_typed_project_with_profile(&typed, options.profile.as_deref())
    }
    .map_err(|error| format!("lower error: {error}"))?;
    let dag_paths = daglang_lower::extract_output_paths(&lowered);
    let annotation_paths = daglang_lower::extract_outputs_annotation(&typed);
    let output_paths = merge_dedup_paths(dag_paths, annotation_paths);

    let derived = derive_artifacts(&lowered).map_err(|error| format!("derive error: {error}"))?;

    let target_module_name = context
        .target_file
        .as_ref()
        .and_then(|tf| {
            let canonical = {
                #[allow(clippy::disallowed_methods)]
                std::fs::canonicalize(tf).ok()
            };
            discover_module_graph_for_context(context)
                .ok()?
                .modules
                .into_iter()
                .find(|m| {
                    m.path == *tf
                        || canonical
                            .as_ref()
                            .is_some_and(|c| m.path == *c)
                })
                .map(|m| m.module_path.join("."))
        });

    let target = options.target;
    let layer = options.layer;
    let mut emitted = emit_with_options(
        &lowered,
        &derived,
        options,
        target_module_name.as_deref(),
        &extern_assets,
    )
    .map_err(|error| format!("emit error: {error}"))?;
    let emit_manifest_path = append_emit_manifest(&mut emitted, target, layer)?;

    let pipeline_params = collect_pipeline_params(&typed);
    let inferred_entrypoints = daglang_lower::infer_entrypoints(&lowered);
    let dsl_type_registry = extract_dsl_type_registry(&typed);

    let receipt = compute_receipt(&lowered, &emitted, &emit_manifest_path, &source_paths);

    Ok(CompileOutput {
        lowered_dag: lowered,
        derived,
        emitted,
        emit_manifest_path,
        output_paths,
        pipeline_params,
        inferred_entrypoints,
        dsl_type_registry,
        receipt,
    })
}

/// Collect extern asset declarations from the typed project.
fn collect_extern_assets(typed: &TypedProject) -> BTreeSet<ProgramSymbolId> {
    let mut assets = BTreeSet::new();
    for module in &typed.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            if let Item::ExternAssetDecl(def) = &item.node {
                assets.insert(ProgramSymbolId::from_parts(&module_name, &def.name));
            }
        }
    }
    assets
}

/// Collect `param` declarations from all modules in the typed project.
fn collect_pipeline_params(typed: &TypedProject) -> Vec<PipelineParam> {
    let mut params = Vec::new();
    for module in &typed.modules {
        for item in &module.ast.items {
            if let Item::ParamDecl(decl) = &item.node {
                let type_id = type_expr_to_string(&decl.ty);
                let default_value = decl.default.as_ref().and_then(expr_to_default_string);
                params.push(PipelineParam {
                    name: decl.name.clone(),
                    type_id,
                    default_value,
                });
            }
        }
    }
    params
}

/// Extract a `TypeRegistry` from DSL-defined sum and product types.
///
/// Walks all modules in the `TypedProject` and registers:
/// - `TypeBody::Sum(variants)` → `type_lib::coproduct(name, variants)`
/// - `TypeBody::Record(fields)` → `type_lib::product(name, fields)`
///
/// This makes DSL-defined types (e.g., `EntryKind`, `AuthScheme`) visible
/// to testgen for variant coverage obligations.
fn extract_dsl_type_registry(typed: &TypedProject) -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    for module in &typed.modules {
        for item in &module.ast.items {
            if let Item::TypeDef(def) = &item.node {
                match &def.body {
                    TypeBody::Sum(variants) => {
                        let variant_pairs: Vec<(&str, &str)> = variants
                            .iter()
                            .map(|v| (v.name.as_str(), "String"))
                            .collect();
                        registry.register(
                            def.name.as_str(),
                            gunbc_ir::type_lib::coproduct(def.name.as_str(), variant_pairs),
                        );
                    }
                    TypeBody::Record(fields) => {
                        let field_type_strings: Vec<(String, String)> = fields
                            .iter()
                            .map(|f| (f.name.clone(), type_expr_to_string(&f.ty)))
                            .collect();
                        let field_pairs: Vec<(&str, &str)> = field_type_strings
                            .iter()
                            .map(|(n, t)| (n.as_str(), t.as_str()))
                            .collect();
                        registry.register(
                            def.name.as_str(),
                            gunbc_ir::type_lib::product(def.name.as_str(), field_pairs),
                        );
                    }
                    TypeBody::Alias(_) => {}
                }
            }
        }
    }
    registry
}

/// Convert a literal default expression to its string representation.
fn expr_to_default_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(s)) => Some(s.clone()),
        Expr::Literal(Literal::Int(n)) => Some(n.to_string()),
        Expr::Literal(Literal::Bool(b)) => Some(b.to_string()),
        Expr::Literal(Literal::Float(f)) => Some(f.to_string()),
        _ => None,
    }
}

pub fn check_from_context(context: &DriverContext) -> Result<CheckOutput, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    check_from_module_graph(module_graph)
}

pub fn check_from_module_graph(module_graph: ModuleGraph) -> Result<CheckOutput, CompileError> {
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

/// Extract pipeline `param` declarations from a DSL file without full compilation.
///
/// Parses and typechecks the module graph, then collects all `ParamDecl` items.
/// Lighter weight than `compile_from_context` — no lowering, deriving, or emission.
pub fn load_pipeline_params(context: &DriverContext) -> Result<Vec<PipelineParam>, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    let typed = typecheck_module_graph_with_options(
        module_graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .map_err(format_typecheck_errors)?;
    Ok(collect_pipeline_params(&typed))
}

/// Generate Rust type definitions from DSL TypeDefs in the given modules.
///
/// Typechecks the module graph, then extracts all `TypeDef` items and
/// converts them to Rust struct/enum definitions via `type_codegen`.
pub fn generate_types_from_context(
    context: &DriverContext,
    module_filter: &[&str],
) -> Result<String, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    // Type generation only needs structural defs, not service bindings.
    let typed = typecheck_module_graph_with_options(
        module_graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .map_err(format_typecheck_errors)?;
    Ok(daglang_emit::type_codegen::generate_types_for_modules(
        &typed,
        module_filter,
    ))
}

/// Report-stage coverage lint finding for a single pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportCoverageIssue {
    pub module: String,
    pub pipeline: String,
    pub declared_stages: Vec<String>,
    pub covered_stages: Vec<String>,
    pub missing_stages: Vec<String>,
}

/// Lint pipeline report stages and verify they reference all declared stages.
///
/// Coverage is inferred structurally from `report` stage expressions by
/// tracking status arguments (`success`/`skipped`) in stage-entry constructor
/// calls and mapping referenced variables back to the stage that produced them.
pub fn lint_report_coverage_from_context(
    context: &DriverContext,
) -> Result<Vec<ReportCoverageIssue>, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    let typed = typecheck_module_graph_with_options(
        module_graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .map_err(format_typecheck_errors)?;
    Ok(lint_report_coverage(&typed))
}

fn lint_report_coverage(typed: &TypedProject) -> Vec<ReportCoverageIssue> {
    let mut issues = Vec::new();

    for module in &typed.modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::PipelineDef(def) = &item.node else {
                continue;
            };
            let Some(report_stage) = def.stages.iter().find(|stage| stage.name == "report") else {
                continue;
            };
            let declared_stages = def
                .stages
                .iter()
                .map(|stage| stage.name.clone())
                .filter(|name| name != "report")
                .collect::<Vec<_>>();
            if declared_stages.is_empty() {
                continue;
            }
            let producer_by_binding = collect_stage_binding_producers(def);
            let covered_set = collect_covered_stages(report_stage, &producer_by_binding);
            if covered_set.is_empty() {
                continue;
            }
            let covered_stages = declared_stages
                .iter()
                .filter(|name| covered_set.contains(name.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let missing_stages = declared_stages
                .iter()
                .filter(|name| !covered_set.contains(name.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if missing_stages.is_empty() {
                continue;
            }
            issues.push(ReportCoverageIssue {
                module: module_name.clone(),
                pipeline: def.name.clone(),
                declared_stages,
                covered_stages,
                missing_stages,
            });
        }
    }

    issues
}

fn collect_stage_binding_producers(def: &PipelineDef) -> HashMap<String, String> {
    let mut by_binding = HashMap::new();
    for stage in &def.stages {
        for stmt in &stage.body.stmts {
            match stmt {
                Stmt::Let(name, _) | Stmt::Assign(name, _) => {
                    by_binding.insert(name.clone(), stage.name.clone());
                }
                Stmt::Annotation(_) | Stmt::Expr(_) | Stmt::Return(_) => {}
            }
        }
    }
    by_binding
}

fn collect_covered_stages(
    report_stage: &StageDef,
    producer_by_binding: &HashMap<String, String>,
) -> std::collections::BTreeSet<String> {
    let mut covered = std::collections::BTreeSet::new();
    for stmt in &report_stage.body.stmts {
        collect_covered_stages_from_stmt(stmt, producer_by_binding, &mut covered);
    }
    covered
}

fn collect_covered_stages_from_stmt(
    stmt: &Stmt,
    producer_by_binding: &HashMap<String, String>,
    covered: &mut std::collections::BTreeSet<String>,
) {
    match stmt {
        Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
            collect_covered_stages_from_expr(expr, producer_by_binding, covered);
        }
        Stmt::Return(fields) => {
            for (_name, expr) in fields {
                collect_covered_stages_from_expr(expr, producer_by_binding, covered);
            }
        }
        Stmt::Annotation(_) => {}
    }
}

fn collect_covered_stages_from_expr(
    expr: &Expr,
    producer_by_binding: &HashMap<String, String>,
    covered: &mut std::collections::BTreeSet<String>,
) {
    match expr {
        Expr::Literal(_) | Expr::Ident(_) => {}
        Expr::FieldAccess(base, _) => {
            collect_covered_stages_from_expr(base, producer_by_binding, covered);
        }
        Expr::Call(_, args) => {
            for (name, arg_expr) in args {
                if matches!(name.as_deref(), Some("success") | Some("skipped")) {
                    let mut roots = std::collections::BTreeSet::new();
                    collect_root_identifiers(arg_expr, &mut roots);
                    for root in roots {
                        if let Some(stage) = producer_by_binding.get(&root) {
                            covered.insert(stage.clone());
                        }
                    }
                }
                collect_covered_stages_from_expr(arg_expr, producer_by_binding, covered);
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_name, arg_expr) in args {
                collect_covered_stages_from_expr(arg_expr, producer_by_binding, covered);
            }
        }
        Expr::BinOp(lhs, _, rhs) => {
            collect_covered_stages_from_expr(lhs, producer_by_binding, covered);
            collect_covered_stages_from_expr(rhs, producer_by_binding, covered);
        }
        Expr::UnaryOp(_, inner) => {
            collect_covered_stages_from_expr(inner, producer_by_binding, covered);
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_covered_stages_from_expr(inner, producer_by_binding, covered);
                }
            }
        }
        Expr::Record(_, fields) => {
            for (_name, value) in fields {
                collect_covered_stages_from_expr(value, producer_by_binding, covered);
            }
        }
        Expr::Match(target, arms) => {
            collect_covered_stages_from_expr(target, producer_by_binding, covered);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_covered_stages_from_expr(guard, producer_by_binding, covered);
                }
                collect_covered_stages_from_expr(&arm.body, producer_by_binding, covered);
            }
        }
        Expr::If(condition, then_expr, else_expr) => {
            collect_covered_stages_from_expr(condition, producer_by_binding, covered);
            collect_covered_stages_from_expr(then_expr, producer_by_binding, covered);
            if let Some(else_expr) = else_expr {
                collect_covered_stages_from_expr(else_expr, producer_by_binding, covered);
            }
        }
        Expr::For(_element, iterable, _passthrough, body) => {
            collect_covered_stages_from_expr(iterable, producer_by_binding, covered);
            collect_covered_stages_from_expr(body, producer_by_binding, covered);
        }
        Expr::Pipe(lhs, rhs) => {
            collect_covered_stages_from_expr(lhs, producer_by_binding, covered);
            collect_covered_stages_from_expr(rhs, producer_by_binding, covered);
        }
        Expr::Lambda(_, body) => {
            collect_covered_stages_from_expr(body, producer_by_binding, covered);
        }
        Expr::List(items) => {
            for item in items {
                collect_covered_stages_from_expr(item, producer_by_binding, covered);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_covered_stages_from_expr(key, producer_by_binding, covered);
                collect_covered_stages_from_expr(value, producer_by_binding, covered);
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_covered_stages_from_expr(inner, producer_by_binding, covered);
            collect_covered_stages_from_expr(guard, producer_by_binding, covered);
        }
        Expr::After(inner, _) => {
            collect_covered_stages_from_expr(inner, producer_by_binding, covered);
        }
        Expr::Return(fields) => {
            for (_name, value) in fields {
                collect_covered_stages_from_expr(value, producer_by_binding, covered);
            }
        }
    }
}

fn collect_root_identifiers(expr: &Expr, roots: &mut std::collections::BTreeSet<String>) {
    match expr {
        Expr::Ident(name) => {
            roots.insert(name.clone());
        }
        Expr::FieldAccess(base, _) => match base.as_ref() {
            Expr::Ident(name) => {
                roots.insert(name.clone());
            }
            other => collect_root_identifiers(other, roots),
        },
        Expr::Call(_, args) | Expr::ServiceCall(_, args) => {
            for (_name, arg_expr) in args {
                collect_root_identifiers(arg_expr, roots);
            }
        }
        Expr::BinOp(lhs, _, rhs) => {
            collect_root_identifiers(lhs, roots);
            collect_root_identifiers(rhs, roots);
        }
        Expr::UnaryOp(_, inner) => collect_root_identifiers(inner, roots),
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_root_identifiers(inner, roots);
                }
            }
        }
        Expr::Record(_, fields) | Expr::Return(fields) => {
            for (_name, value) in fields {
                collect_root_identifiers(value, roots);
            }
        }
        Expr::Match(target, arms) => {
            collect_root_identifiers(target, roots);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_root_identifiers(guard, roots);
                }
                collect_root_identifiers(&arm.body, roots);
            }
        }
        Expr::If(condition, then_expr, else_expr) => {
            collect_root_identifiers(condition, roots);
            collect_root_identifiers(then_expr, roots);
            if let Some(else_expr) = else_expr {
                collect_root_identifiers(else_expr, roots);
            }
        }
        Expr::For(_element, iterable, _passthrough, body) => {
            collect_root_identifiers(iterable, roots);
            collect_root_identifiers(body, roots);
        }
        Expr::Pipe(lhs, rhs) => {
            collect_root_identifiers(lhs, roots);
            collect_root_identifiers(rhs, roots);
        }
        Expr::Lambda(_, body) => collect_root_identifiers(body, roots),
        Expr::List(items) => {
            for item in items {
                collect_root_identifiers(item, roots);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_root_identifiers(key, roots);
                collect_root_identifiers(value, roots);
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_root_identifiers(inner, roots);
            collect_root_identifiers(guard, roots);
        }
        Expr::After(inner, _) => collect_root_identifiers(inner, roots),
        Expr::Literal(_) => {}
    }
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
    target_module_name: Option<&str>,
    extern_assets: &BTreeSet<ProgramSymbolId>,
) -> Result<EmissionBundle, CompileError> {
    match (options.target, options.layer) {
        (CodegenTarget::Rust, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_rust_bundle(&reachable, derived).map_err(|error| {
                CompileError::from(format!("rust emit backend failed: {error}"))
            })
        }
        (CodegenTarget::Go, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_go_bundle(&reachable, derived, extern_assets, &options.embedded_data)
                .map_err(|error| CompileError::from(format!("go emit backend failed: {error}")))
        }
        (CodegenTarget::C, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_c_bundle(&reachable, derived, extern_assets, &options.embedded_data)
                .map_err(|error| CompileError::from(format!("c emit backend failed: {error}")))
        }
        (CodegenTarget::Mips, CodegenLayer::Native) => {
            let reachable = ReachableDag::from_dag(dag);
            emit_mips_bundle(&reachable, derived, extern_assets, &options.embedded_data)
                .map_err(|error| CompileError::from(format!("mips emit backend failed: {error}")))
        }
        (CodegenTarget::Rust, CodegenLayer::ExecRuntime) => {
            let module_name = target_module_name
                .or_else(|| {
                    derived
                        .tool_metadata
                        .modules
                        .first()
                        .map(|module| module.module.as_str())
                })
                .unwrap_or("daglang.generated");
            let files = emit_exec_runtime_with_output_dir(dag, module_name, options.output_dir.as_deref())
                .map_err(|error| {
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

#[derive(Debug, Serialize)]
struct EmitManifestDocument {
    backend: String,
    target: String,
    layer: String,
    files: Vec<EmitManifestEntry>,
}

#[derive(Debug, Serialize)]
struct EmitManifestEntry {
    path: String,
    bytes: usize,
    sha256: String,
}

fn append_emit_manifest(
    emitted: &mut EmissionBundle,
    target: CodegenTarget,
    layer: CodegenLayer,
) -> Result<String, CompileError> {
    let manifest_path = emit_manifest_path(target, layer);
    let mut files = emitted
        .files
        .iter()
        .filter(|file| file.path != manifest_path)
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let entries = files
        .iter()
        .map(|file| EmitManifestEntry {
            path: file.path.clone(),
            bytes: file.content.len(),
            sha256: sha256_hex(file.content.as_bytes()),
        })
        .collect::<Vec<_>>();

    let manifest = EmitManifestDocument {
        backend: emitted.backend.clone(),
        target: target.to_string(),
        layer: layer.to_string(),
        files: entries,
    };
    let content = serde_json::to_string_pretty(&manifest).map_err(|error| {
        CompileError::from(format!("failed to serialize emit manifest: {error}"))
    })?;
    emitted.files.push(EmittedFile {
        path: manifest_path.clone(),
        content,
    });
    Ok(manifest_path)
}

fn emit_manifest_path(target: CodegenTarget, layer: CodegenLayer) -> String {
    match layer {
        CodegenLayer::Native => format!("target/generated/{target}/emit_manifest.json"),
        CodegenLayer::ExecRuntime => "emit_manifest.json".to_string(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Compute a deterministic compilation receipt from the compilation artifacts.
///
/// The receipt contains content-addressable digests for source files, the
/// canonical IR, and the emit manifest. Two compilations of the same input
/// MUST produce identical receipts — this is the determinism contract.
fn compute_receipt(
    dag: &Dag<LoweredOp>,
    emitted: &EmissionBundle,
    emit_manifest_path: &str,
    source_paths: &[PathBuf],
) -> Option<CompileReceipt> {
    // Source digest: sha256 of sorted per-file content hashes.
    let mut source_hashes: Vec<String> = Vec::with_capacity(source_paths.len());
    for path in source_paths {
        #[allow(clippy::disallowed_methods)]
        let content = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(_) => return None,
        };
        source_hashes.push(sha256_hex(&content));
    }
    source_hashes.sort();
    let source_digest = sha256_hex(source_hashes.join("\n").as_bytes());

    // Program IR digest: sha256 of canonical IR JSON.
    let canonical_json = match daglang_lower::canonical_ir_json(dag) {
        Ok(json) => json,
        Err(_) => return None,
    };
    let program_ir_digest = sha256_hex(canonical_json.as_bytes());

    // Emit manifest digest: sha256 of the manifest file content.
    let emit_manifest_digest = emitted
        .files
        .iter()
        .find(|f| f.path == emit_manifest_path)
        .map(|f| sha256_hex(f.content.as_bytes()))
        .unwrap_or_default();

    Some(CompileReceipt {
        source_digest,
        program_ir_digest,
        emit_manifest_digest,
    })
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

fn include_profile_modules(
    module_graph: &mut ModuleGraph,
    roots: &[PathBuf],
    include_bound_services: bool,
) -> Result<(), CompileError> {
    let mut seed_files = Vec::<PathBuf>::new();
    for root in roots {
        let discovery_root = root.join("profiles");
        if !discovery_root.is_dir() {
            continue;
        }
        let mut discovered =
            daglang_resolve::discover_dag_files(&discovery_root).map_err(format_resolve_error)?;
        seed_files.append(&mut discovered);
    }
    seed_files.sort();
    seed_files.dedup();
    if seed_files.is_empty() {
        return Ok(());
    }

    if include_bound_services {
        let canonical_roots = daglang_resolve::canonicalize_roots(roots);
        let mut implementation_modules = Vec::<PathBuf>::new();
        for profile_file in &seed_files {
            let (module, _imports) =
                parse_target_module_file(profile_file, roots, &canonical_roots)?;
            for item in &module.ast.items {
                let Item::ProfileDef(def) = &item.node else {
                    continue;
                };
                for bind in &def.binds {
                    if let Some(path) = resolve_profile_bind_implementation_module_path(
                        roots,
                        &bind.implementation_type,
                    ) {
                        implementation_modules.push(path);
                    }
                }
            }
        }
        seed_files.append(&mut implementation_modules);
        seed_files.sort();
        seed_files.dedup();
    }

    let canonical_roots = daglang_resolve::canonicalize_roots(roots);
    let mut module_index_by_path = module_graph
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.path.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut module_index_by_decl = module_graph
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut pending = VecDeque::from(seed_files);
    let mut queued_paths = HashSet::<PathBuf>::new();
    while let Some(file) = pending.pop_front() {
        if !queued_paths.insert(file.clone()) {
            continue;
        }
        let (module, imports) = parse_target_module_file(&file, roots, &canonical_roots)?;
        if let Some(existing_index) = module_index_by_path.get(&module.path).copied() {
            let existing = &module_graph.modules[existing_index];
            if existing.module_path != module.module_path {
                return Err(format_resolve_error(ResolveError::DuplicateModule(
                    module.module_path,
                )));
            }
        } else if let Some(existing_index) = module_index_by_decl.get(&module.module_path).copied()
        {
            let existing = &module_graph.modules[existing_index];
            if existing.path != module.path {
                return Err(format_resolve_error(ResolveError::DuplicateModule(
                    module.module_path,
                )));
            }
        } else {
            let mut module = module;
            module.dependencies.clear();
            let next_index = module_graph.modules.len();
            module_index_by_path.insert(module.path.clone(), next_index);
            module_index_by_decl.insert(module.module_path.clone(), next_index);
            module_graph.modules.push(module);
        }
        for import in imports {
            if module_index_by_decl.contains_key(&import) {
                continue;
            }
            if let Some(import_file) = resolve_import_file_path(roots, &import) {
                pending.push_back(import_file);
            }
        }
    }

    let module_index_by_decl = module_graph
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.clone(), index))
        .collect::<HashMap<_, _>>();
    for module in &mut module_graph.modules {
        let mut dependencies = module
            .ast
            .imports
            .iter()
            .filter_map(|import| {
                module_index_by_decl
                    .get(&import.node.path.segments)
                    .copied()
            })
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        dependencies.dedup();
        module.dependencies = dependencies;
    }

    Ok(())
}

fn resolve_profile_bind_implementation_module_path(
    roots: &[PathBuf],
    implementation_type: &str,
) -> Option<PathBuf> {
    let segments = implementation_type
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }
    for end in (1..segments.len()).rev() {
        if let Some(path) = resolve_import_file_path(roots, &segments[..end]) {
            return Some(path);
        }
    }
    None
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
    let result = roots
        .iter()
        .map(|root| root.join(&relative))
        .find(|candidate| candidate.is_file());
    let _ = result;
    result
}

fn callable_scope_for_context(
    context: &DriverContext,
    module_graph: &ModuleGraph,
) -> Result<Option<(HashSet<String>, String)>, CompileError> {
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
    let entry_module_name = target_module.module_path.join(".");
    let mut scope = HashSet::new();
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
        scope.insert(entry_module_name.clone());
    }
    Ok(Some((scope, entry_module_name)))
}

fn module_has_callable_items(module: &ResolvedModule) -> bool {
    module.ast.items.iter().any(|item| {
        matches!(
            item.node,
            Item::FnDef(_) | Item::FuncDef(_) | Item::PatternDef(_) | Item::PipelineDef(_)
        )
    })
}

/// Merge two sorted path lists into one sorted, deduplicated list.
fn merge_dedup_paths(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let mut set: std::collections::BTreeSet<String> = a.into_iter().collect();
    set.extend(b);
    set.into_iter().collect()
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
                    inferred_segments.push(part.to_string_lossy().into_owned());
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

// ── Inline test extraction ──────────────────────────────────────────

/// Result of extracting and emitting inline tests from compiled modules.
#[derive(Debug)]
pub struct TestEmitOutput {
    /// Map from module source path to generated Rust test code.
    pub generated: Vec<(PathBuf, String)>,
}

/// Extract inline tests from an already-resolved module graph.
///
/// Tests and fixtures are defined directly within `.dag` files alongside
/// the tool/service definitions they test. This function walks the module
/// graph, extracts any `test` / `fixture` blocks from each module's AST,
/// and emits the corresponding `graph_mock.rs` Rust code.
///
/// The `config_resolver` receives the module path segments (e.g.
/// `["tools", "bootstrap"]`) and the filesystem path, returning the
/// `TestEmitConfig` when the module has a registered test target.
pub fn extract_inline_tests(
    graph: &daglang_resolve::ModuleGraph,
    config_resolver: impl Fn(&[String], &Path) -> Option<daglang_emit::test_mock_emit::TestEmitConfig>,
) -> Result<TestEmitOutput, CompileError> {
    let mut generated = Vec::new();

    for module in &graph.modules {
        let test_file = daglang_emit::test_mock_emit::TestFile::from_source(&module.ast);

        if test_file.tests.is_empty() {
            continue; // Module has no inline tests.
        }

        let config = config_resolver(&module.module_path, &module.path).ok_or_else(|| {
            format!(
                "no test emit config for module `{}`",
                module.module_path.join(".")
            )
        })?;

        let rust_code = daglang_emit::test_mock_emit::emit_test_mock_file(&test_file, &config);
        generated.push((module.path.clone(), rust_code));
    }

    Ok(TestEmitOutput { generated })
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
    fn report_coverage_lint_passes_when_report_references_all_stages() {
        let root = unique_temp_dir("report_coverage_pass");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(
            &file,
            r#"module sample
fn report_entry(name: String, success: Bool) -> Bool { success }
pipeline ci {
  stage codegen { codegen_ok = true }
  stage test [after codegen] { test_ok = true }
  stage report [after test] {
    entries = [
      report_entry(name: "codegen", success: codegen_ok),
      report_entry(name: "test", success: test_ok)
    ]
  }
}
"#,
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let issues = lint_report_coverage_from_context(&context)
            .expect("report coverage lint should compile and run");
        assert!(
            issues.is_empty(),
            "all stages were referenced in report; issues: {issues:?}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_coverage_lint_reports_missing_stage_references() {
        let root = unique_temp_dir("report_coverage_fail");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(
            &file,
            r#"module sample
fn report_entry(name: String, success: Bool) -> Bool { success }
pipeline ci {
  stage codegen { codegen_ok = true }
  stage test [after codegen] { test_ok = true }
  stage report [after test] {
    entries = [report_entry(name: "codegen", success: codegen_ok)]
  }
}
"#,
        )
        .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let issues = lint_report_coverage_from_context(&context)
            .expect("report coverage lint should compile and run");
        assert_eq!(issues.len(), 1, "expected one missing-stage issue");
        let issue = &issues[0];
        assert_eq!(issue.pipeline, "ci");
        assert!(
            issue.missing_stages.contains(&"test".to_string()),
            "expected `test` stage to be reported missing: {issue:?}"
        );
        assert!(
            issue.covered_stages.contains(&"codegen".to_string()),
            "expected `codegen` stage to be covered: {issue:?}"
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
    fn compile_single_function_target_includes_callable_dependency_closure() {
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
            node_ids.contains("sample.helper::dep_task"),
            "function single-file compile should include callable dependencies"
        );
        assert!(node_ids.contains("sample.main::run"));

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_bound_interface_requires_profile_flag() {
        let root = unique_temp_dir("compile_profile_required");
        std::fs::create_dir_all(root.join("pipelines")).expect("failed to create pipelines dir");
        std::fs::create_dir_all(root.join("interfaces")).expect("failed to create interfaces dir");
        std::fs::create_dir_all(root.join("services")).expect("failed to create services dir");
        std::fs::create_dir_all(root.join("profiles")).expect("failed to create profiles dir");
        std::fs::write(
            root.join("interfaces/issue_provider.dag"),
            r#"module interfaces.issue_provider
interface IssueProvider {
  capability get {
    input {}
    output { ok: Bool }
  }
}"#,
        )
        .expect("failed to write interface source");
        std::fs::write(
            root.join("services/stub_provider.dag"),
            r#"module services.stub_provider
import interfaces.issue_provider { IssueProvider }
service stub.Provider : IssueProvider {
  operation get {
    input {}
    output { ok: Bool }
    @rest(GET, "/ok")
  }
}"#,
        )
        .expect("failed to write service source");
        std::fs::write(
            root.join("profiles/sdlc.dag"),
            r#"module profiles.sdlc
profile unit_test {
  bind IssueProvider -> services.stub_provider.stub.Provider
}"#,
        )
        .expect("failed to write profile source");
        let file = root.join("pipelines/main.dag");
        std::fs::write(
            &file,
            r#"module pipelines.main
import interfaces.issue_provider { IssueProvider }
func run() -> { ok: Bool } uses issues: IssueProvider {
  result = issues.get()
  return { ok: result.ok }
}"#,
        )
        .expect("failed to write pipeline source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        // IS-3: Compilation without --profile now succeeds with stub interfaces
        // instead of hard-erroring. The resulting DAG is valid for DryRun testing.
        let output = compile_from_context(&context)
            .expect("compile should succeed with stub interfaces (IS-3)");
        assert!(
            !output.lowered_dag.nodes.is_empty(),
            "compilation without profile should produce a non-empty DAG with stub transport"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_single_file_with_profile_loads_profiles_from_root() {
        let root = unique_temp_dir("compile_profile_loading");
        std::fs::create_dir_all(root.join("pipelines")).expect("failed to create pipelines dir");
        std::fs::create_dir_all(root.join("interfaces")).expect("failed to create interfaces dir");
        std::fs::create_dir_all(root.join("services")).expect("failed to create services dir");
        std::fs::create_dir_all(root.join("profiles")).expect("failed to create profiles dir");
        std::fs::write(
            root.join("interfaces/issue_provider.dag"),
            r#"module interfaces.issue_provider
interface IssueProvider {
  capability get {
    input {}
    output { ok: Bool }
  }
}"#,
        )
        .expect("failed to write interface source");
        std::fs::write(
            root.join("services/stub_provider.dag"),
            r#"module services.stub_provider
import interfaces.issue_provider { IssueProvider }
service stub.Provider : IssueProvider {
  operation get {
    input {}
    output { ok: Bool }
    @rest(GET, "/ok")
  }
}"#,
        )
        .expect("failed to write service source");
        std::fs::write(
            root.join("profiles/sdlc.dag"),
            r#"module profiles.sdlc
profile unit_test {
  bind IssueProvider -> services.stub_provider.stub.Provider
}"#,
        )
        .expect("failed to write profile source");
        let file = root.join("pipelines/main.dag");
        std::fs::write(
            &file,
            r#"module pipelines.main
import interfaces.issue_provider { IssueProvider }
func run() -> { ok: Bool } uses issues: IssueProvider {
  result = issues.get()
  return { ok: result.ok }
}"#,
        )
        .expect("failed to write pipeline source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file),
        };
        let output = compile_from_context_with_options(
            &context,
            CompileOptions {
                profile: Some("unit_test".to_string()),
                ..CompileOptions::default()
            },
        )
        .expect("compile should succeed when profile modules are loaded");
        assert!(
            output
                .lowered_dag
                .nodes
                .iter()
                .any(|node| node.id.0 == "pipelines.main::run"),
            "compiled DAG should include target callable"
        );
        assert!(
            output.lowered_dag.edges.iter().any(|edge| {
                edge.to_node.0 == "pipelines.main::run" && edge.to_port.0 == "__deps"
            }),
            "bound service transport edge should feed target callable dependencies"
        );

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

    #[test]
    fn compile_includes_emit_manifest_with_hashes() {
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

        assert_eq!(output.emit_manifest_path, "target/generated/go/emit_manifest.json");
        let manifest = output
            .emitted
            .files
            .iter()
            .find(|file| file.path == output.emit_manifest_path)
            .expect("emit manifest should be present in emitted files");
        let manifest_json: serde_json::Value =
            serde_json::from_str(&manifest.content).expect("emit manifest should be valid JSON");
        let files = manifest_json
            .get("files")
            .and_then(|value| value.as_array())
            .expect("emit manifest should include files array");
        assert!(
            files.iter().any(|entry| {
                entry.get("path").and_then(|v| v.as_str()) == Some("target/generated/go/main.go")
            }),
            "emit manifest should include generated main.go entry"
        );
        assert!(
            files.iter().all(|entry| {
                entry
                    .get("sha256")
                    .and_then(|value| value.as_str())
                    .is_some_and(|hash| hash.len() == 64)
            }),
            "every emit manifest entry should include a 64-char sha256 hash"
        );
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
        // Exec-runtime skips SubDag nodes (e.g. for-loop expansions from
        // unreachable callables), so compare against Opaque-only counts.
        let opaque_nodes: Vec<_> = output
            .lowered_dag
            .nodes
            .iter()
            .filter(|n| matches!(n.body, gunbc_ir::node::NodeBody::Opaque(_)))
            .collect();
        let opaque_ids: std::collections::HashSet<_> =
            opaque_nodes.iter().map(|n| &n.id).collect();
        let expected_nodes = opaque_nodes.len();
        let actual_nodes = main_rs.matches("dag.add_node").count();
        assert_eq!(
            actual_nodes, expected_nodes,
            "generated DAG should have {expected_nodes} nodes, got {actual_nodes}"
        );

        let expected_edges = output
            .lowered_dag
            .edges
            .iter()
            .filter(|e| opaque_ids.contains(&e.from_node) && opaque_ids.contains(&e.to_node))
            .count();
        let actual_edges = main_rs.matches("dag.add_edge").count();
        assert_eq!(
            actual_edges, expected_edges,
            "generated DAG should have {expected_edges} edges, got {actual_edges}"
        );

        for node in &opaque_nodes {
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
        // Exec-runtime skips SubDag nodes, so compare against Opaque-only counts.
        let opaque_nodes: Vec<_> = output
            .lowered_dag
            .nodes
            .iter()
            .filter(|n| matches!(n.body, gunbc_ir::node::NodeBody::Opaque(_)))
            .collect();
        let opaque_ids: std::collections::HashSet<_> =
            opaque_nodes.iter().map(|n| &n.id).collect();
        let expected_nodes = opaque_nodes.len();
        let actual_nodes = main_rs.matches("dag.add_node").count();
        assert_eq!(
            actual_nodes, expected_nodes,
            "generated DAG should have {expected_nodes} nodes, got {actual_nodes}"
        );

        let expected_edges = output
            .lowered_dag
            .edges
            .iter()
            .filter(|e| opaque_ids.contains(&e.from_node) && opaque_ids.contains(&e.to_node))
            .count();
        let actual_edges = main_rs.matches("dag.add_edge").count();
        assert_eq!(
            actual_edges, expected_edges,
            "generated DAG should have {expected_edges} edges, got {actual_edges}"
        );

        for node in &opaque_nodes {
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

    // ---- Determinism contract tests ----

    /// Helper: compile a target and return the receipt, panicking on failure.
    fn compile_with_receipt(root: &Path, target_file: Option<&Path>) -> CompileReceipt {
        let context = DriverContext {
            roots: vec![root.to_path_buf()],
            target_file: target_file.map(|p| p.to_path_buf()),
        };
        let output = compile_from_context(&context)
            .expect("compile should succeed for determinism test");
        output
            .receipt
            .expect("receipt should be computed for determinism test")
    }

    #[test]
    fn determinism_single_file_compile_produces_identical_receipts() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/makegen.dag");

        let receipt_a = compile_with_receipt(&root, Some(&file));
        let receipt_b = compile_with_receipt(&root, Some(&file));

        assert_eq!(
            receipt_a, receipt_b,
            "two compilations of the same single file must produce identical receipts"
        );
        assert!(
            !receipt_a.source_digest.is_empty(),
            "source_digest should be non-empty"
        );
        assert!(
            !receipt_a.program_ir_digest.is_empty(),
            "program_ir_digest should be non-empty"
        );
        assert!(
            !receipt_a.emit_manifest_digest.is_empty(),
            "emit_manifest_digest should be non-empty"
        );
    }

    #[test]
    fn determinism_directory_compile_produces_identical_receipts() {
        let root = unique_temp_dir("determinism_dir");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        std::fs::write(
            root.join("alpha.dag"),
            "module alpha\nfn run() -> Unit {}\n",
        )
        .expect("failed to write alpha");
        std::fs::write(
            root.join("beta.dag"),
            "module beta\nimport alpha\nfn process(input: String) -> String { input }\n",
        )
        .expect("failed to write beta");

        let receipt_a = compile_with_receipt(&root, None);
        let receipt_b = compile_with_receipt(&root, None);

        assert_eq!(
            receipt_a, receipt_b,
            "two compilations of the same directory must produce identical receipts"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn determinism_ci_pipeline_compile_produces_identical_receipts() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("pipelines/ci.dag");

        let receipt_a = compile_with_receipt(&root, Some(&file));
        let receipt_b = compile_with_receipt(&root, Some(&file));

        assert_eq!(
            receipt_a, receipt_b,
            "two compilations of the CI pipeline must produce identical receipts"
        );
    }

    #[test]
    fn determinism_diagnostic_ordering_is_stable() {
        let root = unique_temp_dir("determinism_diag");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        // Create a file with multiple intentional errors.
        std::fs::write(
            root.join("bad.dag"),
            concat!(
                "module bad\n",
                "import nonexistent.alpha\n",
                "import nonexistent.beta\n",
                "import nonexistent.gamma\n",
            ),
        )
        .expect("failed to write source with errors");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(root.join("bad.dag")),
        };
        let error_a = compile_from_context(&context)
            .expect_err("compile should fail for bad source")
            .to_string();
        let error_b = compile_from_context(&context)
            .expect_err("compile should fail for bad source")
            .to_string();

        assert_eq!(
            error_a, error_b,
            "diagnostic output must be byte-identical across compilations"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }
}
