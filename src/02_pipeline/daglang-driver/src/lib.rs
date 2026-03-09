use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write;
use std::path::{Path, PathBuf};

mod pipeline;
mod prepare;
mod receipt;

use daglang_contract::Diagnostics;
use daglang_derive::{derive_artifacts, DeriveError, DerivedArtifacts};
use daglang_emit::{
    emit_c_bundle, emit_go_bundle, emit_mips_bundle, emit_rust_bundle, EmissionBundle,
    EmissionSummary, EmitError, EmittedFile,
};
pub use daglang_lower::is_user_param_port;
pub use daglang_lower::InferredEntrypoint;
use daglang_lower::{lower_to_output_with_config, LowerError, LoweredOp, LoweringConfig};
use daglang_resolve::{ModuleGraph, ResolveError, ResolvedModule};
use daglang_syntax::ast::{Expr, Item, Literal, ModulePath, PipelineDef, StageDef, Stmt};
use daglang_syntax::parser;
pub use daglang_typecheck::PipelineParam;
use daglang_typecheck::{
    typecheck_module_graph_located, typecheck_module_graph_with_options, SpannedTypeError,
    TypeError, TypecheckOptions, TypedProject,
};
use gunbc_ir::{Dag, ProgramSymbolId, ReachableDag, TypeRegistry, VerifiedDag};
use serde::{Deserialize, Serialize};

pub use pipeline::run_compile_pipeline;
pub use prepare::{prepare_compile_context, PreparedCompileContext};
use receipt::sha256_hex;
pub(crate) use receipt::{compute_receipt, compute_source_digest_from_module_graph};
pub use receipt::{compute_source_digest, compute_source_digest_for_context};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverContext {
    pub roots: Vec<PathBuf>,
    pub target_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct CompileOutput {
    pub lowered_dag: VerifiedDag<LoweredOp>,
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
    pub receipt: Option<CompileReceipt>,
    /// Pure `fn` items whose bodies were lossy-parsed (parser recovery).
    ///
    /// Always empty on successful compilation — `run_compile_pipeline`
    /// rejects any compilation where lossy fn bodies are present
    /// (README invariant §7). Retained in the output struct for
    /// diagnostic tooling.
    pub lossy_fn_bodies: Vec<String>,
    /// Data declaration values evaluated at compile time.
    ///
    /// Keys are both qualified (`module.name`) and unqualified (`name`).
    /// Values are the constant expressions from `data` items.
    pub data_values: HashMap<String, serde_json::Value>,
}

impl CompileOutput {
    /// Emit a data-only `.dag` artifact containing compilation metadata.
    ///
    /// Produces a file with:
    /// - `data entrypoints: List<EntrypointInfo> = [...]`
    /// - `data output_paths: List<String> = [...]`
    ///
    /// The output is a valid `.dag` file that can be imported by downstream
    /// DSL modules for introspection.
    pub fn emit_artifact_dag(&self, module_name: &str) -> String {
        use daglang_emit::dag_emit::{emit_data_dag, DataEntry, TypeDef};

        let types = vec![TypeDef {
            name: "EntrypointInfo".to_string(),
            fields: vec![
                ("func_name".to_string(), "String".to_string()),
                ("module".to_string(), "String".to_string()),
                ("node_id".to_string(), "String".to_string()),
            ],
        }];

        let entrypoints_json: Vec<serde_json::Value> = self
            .inferred_entrypoints
            .iter()
            .map(|ep| {
                serde_json::json!({
                    "func_name": ep.func_name,
                    "module": ep.module,
                    "node_id": ep.node_id,
                })
            })
            .collect();

        let output_paths_json: Vec<serde_json::Value> = self
            .output_paths
            .iter()
            .map(|p| serde_json::json!(p))
            .collect();

        let data = vec![
            DataEntry {
                name: "entrypoints".to_string(),
                type_expr: "List<EntrypointInfo>".to_string(),
                value: serde_json::Value::Array(entrypoints_json),
            },
            DataEntry {
                name: "output_paths".to_string(),
                type_expr: "List<String>".to_string(),
                value: serde_json::Value::Array(output_paths_json),
            },
        ];

        emit_data_dag(module_name, &types, &data)
    }

    /// Serialize the lowered DAG to JSON bytes for AOT caching (C28).
    ///
    /// The serialized format includes the full `Dag<LoweredOp>` with all
    /// `ServiceCallMetadata`, `ServiceOperationSpec`, fn bodies, and pattern ops.
    /// Deserialize with [`deserialize_lowered_dag`].
    pub fn serialize_lowered_dag(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self.lowered_dag)
    }
}

/// Deserialize a lowered DAG from JSON bytes (C28 AOT cache).
///
/// Reconstitutes a `Dag<LoweredOp>` previously serialized by
/// [`CompileOutput::serialize_lowered_dag`]. The caller must then resolve
/// the lowered ops to `DynOp` via `resolve_lowered_dag_with(...)`.
pub fn deserialize_lowered_dag(bytes: &[u8]) -> Result<Dag<LoweredOp>, serde_json::Error> {
    serde_json::from_slice(bytes)
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

/// Structured compiler error preserving phase-specific context.
///
/// Stage-local type/lower/verify failures are normalized into the shared
/// diagnostic contract before they leave the driver.
#[derive(Debug)]
pub enum CompileError {
    Resolve(ResolveError),
    Diagnostics(Diagnostics),
    Emit(EmitError),
    Derive(DeriveError),
    /// Ad-hoc error message (validation, I/O, configuration).
    Message(String),
}

impl CompileError {
    /// Check whether the formatted error message contains the given substring.
    pub fn contains(&self, needle: &str) -> bool {
        self.to_string().contains(needle)
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Resolve(ResolveError::ParseErrors(files)) => {
                f.write_str("compile diagnostics:\n")?;
                let diagnostics = daglang_syntax::diagnostic::normalize_diagnostics(
                    files
                        .iter()
                        .flat_map(|(_path, diagnostics)| diagnostics.clone())
                        .collect(),
                );
                for d in diagnostics {
                    writeln!(f, "  {}", d.render())?;
                }
                Ok(())
            }
            CompileError::Resolve(error) => write!(f, "resolve error: {error}"),
            CompileError::Diagnostics(errors) => {
                f.write_str("compile diagnostics:\n")?;
                for error in &errors.errors {
                    writeln!(f, "  {error}")?;
                }
                Ok(())
            }
            CompileError::Emit(error) => write!(f, "emit error: {error}"),
            CompileError::Derive(error) => write!(f, "derive error: {error}"),
            CompileError::Message(message) => f.write_str(message),
        }
    }
}

impl From<ResolveError> for CompileError {
    fn from(error: ResolveError) -> Self {
        CompileError::Resolve(error)
    }
}

impl From<Vec<TypeError>> for CompileError {
    fn from(errors: Vec<TypeError>) -> Self {
        CompileError::Diagnostics(typecheck_diagnostics(errors))
    }
}

// Note: SpannedTypeError → CompileError conversion now requires the module graph
// for source-text resolution. Use typecheck_diagnostics_located() directly at
// call sites instead of From.

impl From<LowerError> for CompileError {
    fn from(error: LowerError) -> Self {
        CompileError::Diagnostics(lower_diagnostics(error))
    }
}

impl From<Vec<gunbc_ir::VerifyError>> for CompileError {
    fn from(errors: Vec<gunbc_ir::VerifyError>) -> Self {
        CompileError::Diagnostics(verification_diagnostics(errors))
    }
}

impl From<EmitError> for CompileError {
    fn from(error: EmitError) -> Self {
        CompileError::Emit(error)
    }
}

impl From<DeriveError> for CompileError {
    fn from(error: DeriveError) -> Self {
        CompileError::Derive(error)
    }
}

impl From<String> for CompileError {
    fn from(message: String) -> Self {
        CompileError::Message(message)
    }
}

impl From<&str> for CompileError {
    fn from(message: &str) -> Self {
        CompileError::Message(message.to_string())
    }
}

fn typecheck_diagnostics(errors: Vec<TypeError>) -> Diagnostics {
    Diagnostics {
        errors: errors
            .into_iter()
            .map(|error| error.to_diagnostic())
            .collect(),
    }
}

fn typecheck_diagnostics_located(
    errors: Vec<SpannedTypeError>,
    graph: &daglang_resolve::ModuleGraph,
) -> Diagnostics {
    Diagnostics {
        errors: errors
            .into_iter()
            .map(|se| {
                // Look up source text for this module to resolve line:col
                let source = graph
                    .modules
                    .iter()
                    .find(|m| m.path == se.file)
                    .map(|m| m.source.as_str());
                match source {
                    Some(src) => se.to_diagnostic_with_source(src),
                    None => se.to_diagnostic(),
                }
            })
            .collect(),
    }
}

fn lower_diagnostics(error: LowerError) -> Diagnostics {
    Diagnostics::single(error.to_diagnostic())
}

fn verification_diagnostics(errors: Vec<gunbc_ir::VerifyError>) -> Diagnostics {
    Diagnostics {
        errors: errors
            .into_iter()
            .map(|error| error.to_diagnostic())
            .collect(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckOutput {
    pub parsed_files: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

/// Lightweight data extraction from embedded source strings (permissive mode).
///
/// Builds a module graph from `(virtual_path, source_text)` pairs, typechecks
/// with `allow_unresolved_imports: true`, lowers, and extracts data values
/// and fn bodies. No filesystem access.
///
/// **This is a utility path, not mainline compilation.** It uses permissive
/// typechecking and does not validate lossy bodies. It returns
/// `EmbeddedCompileOutput` (not `CompileOutput`/`VerifiedDag`) and cannot
/// masquerade as a successful full compilation.
///
/// Used by build-time generators that embed `.dag` sources via `include_str!`.
pub fn compile_data_from_sources_permissive(
    sources: &[(&Path, &str)],
) -> Result<EmbeddedCompileOutput, CompileError> {
    let mut parsed = Vec::new();
    for (path, source) in sources {
        let ast = parser::parse_with_file_diagnostics(path, source).map_err(|diagnostics| {
            CompileError::Message(format!(
                "failed to parse embedded module {}: {}",
                path.display(),
                diagnostics
                    .iter()
                    .map(|d| d.render())
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        })?;
        let module_path = ast
            .module_path
            .as_ref()
            .map(|mp| mp.node.clone())
            .ok_or_else(|| {
                CompileError::Message(format!(
                    "embedded module {} missing `module` declaration",
                    path.display()
                ))
            })?;
        let imports: Vec<ModulePath> = ast
            .imports
            .iter()
            .map(|imp| imp.node.path.clone())
            .collect();
        parsed.push((
            path.to_path_buf(),
            module_path,
            imports,
            ast,
            source.to_string(),
        ));
    }

    let mut index_by_module = HashMap::new();
    for (idx, (_, module_path, _, _, _)) in parsed.iter().enumerate() {
        index_by_module.insert(module_path.clone(), idx);
    }

    let mut resolved = Vec::new();
    for (path, module_path, imports, ast, source) in parsed {
        let mut dependencies = Vec::new();
        for import in &imports {
            if let Some(&dep) = index_by_module.get(import) {
                dependencies.push(dep);
            }
            // Unresolved imports are tolerated for self-contained data modules.
        }
        resolved.push(ResolvedModule {
            path,
            ast,
            module_path,
            dependencies,
            source,
        });
    }

    let module_graph = ModuleGraph { modules: resolved };
    let typed = typecheck_module_graph_with_options(
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .map_err(CompileError::from)?;
    let mut fns = HashMap::new();
    let lower_output = daglang_lower::lower_to_output_with_config(
        &typed,
        &daglang_lower::LoweringConfig {
            allow_empty_dag: true,
            ..Default::default()
        },
    )
    .map_err(CompileError::from)?;
    extract_fn_bodies_from_dag(&lower_output.dag, &mut fns);
    let data_values = lower_output.data_values;

    let pipelines = extract_pipelines_from_typed(&typed);

    Ok(EmbeddedCompileOutput {
        data_values,
        fns,
        pipelines,
    })
}

/// Filesystem-based data extraction from a DSL module path (permissive mode).
///
/// Discovers the module's transitive imports from the filesystem (no `include_str!`
/// needed), typechecks with `allow_unresolved_imports: true`, lowers, and extracts
/// data values and fn bodies.
///
/// **This is a utility path, not mainline compilation.** It uses permissive
/// typechecking and does not validate lossy bodies. It returns
/// `EmbeddedCompileOutput` (not `CompileOutput`/`VerifiedDag`) and cannot
/// masquerade as a successful full compilation.
///
/// # Arguments
/// * `dsl_root` — Root of the DSL source tree (e.g., `workspace_root.join("dsl")`)
/// * `module_path` — Relative path within `dsl_root` (e.g., `"config/codegen_paths.dag"`)
pub fn compile_data_from_module_permissive(
    dsl_root: &Path,
    module_path: &str,
) -> Result<EmbeddedCompileOutput, CompileError> {
    let target_file = dsl_root.join(module_path);
    let context = DriverContext {
        roots: vec![dsl_root.to_path_buf()],
        target_file: Some(target_file),
    };
    let module_graph = discover_module_graph_for_context(&context)?;
    let typed = typecheck_module_graph_with_options(
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .map_err(CompileError::from)?;
    let mut fns = HashMap::new();
    // Derive the module dotted path from the file path for entry_module scoping.
    // This prevents lowering unrelated callables from transitively imported modules
    // (e.g., credential_chain in std/patterns.dag when only makegen.dag is needed).
    let entry_module = module_path
        .strip_suffix(".dag")
        .unwrap_or(module_path)
        .replace('/', ".");
    let lower_config = daglang_lower::LoweringConfig {
        entry_module: Some(&entry_module),
        allow_empty_dag: true,
        ..Default::default()
    };
    let lower_output = daglang_lower::lower_to_output_with_config(&typed, &lower_config)
        .map_err(CompileError::from)?;
    extract_fn_bodies_from_dag(&lower_output.dag, &mut fns);
    let data_values = lower_output.data_values;

    let pipelines = extract_pipelines_from_typed(&typed);

    Ok(EmbeddedCompileOutput {
        data_values,
        fns,
        pipelines,
    })
}

/// Extract fn bodies from a lowered DAG.
fn extract_fn_bodies_from_dag(
    dag: &gunbc_ir::Dag<daglang_lower::LoweredOp>,
    fns: &mut HashMap<String, daglang_lower::LoweredFnBody>,
) {
    use daglang_lower::{CallableKind, LoweredOp};
    for node in &dag.nodes {
        if let gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable {
            kind: CallableKind::Fn,
            name,
            fn_body: Some(body),
            ..
        }) = &node.body
        {
            fns.insert(name.clone(), *body.clone());
        }
    }
}

/// Walk all modules in a typed project and extract pipeline definitions.
fn extract_pipelines_from_typed(
    typed: &TypedProject<'_>,
) -> HashMap<String, Vec<PipelineStageInfo>> {
    let mut pipelines = HashMap::new();
    for module in typed.modules() {
        for item in &module.ast.items {
            if let Item::PipelineDef(def) = &item.node {
                let stages = def
                    .stages
                    .iter()
                    .map(|stage| PipelineStageInfo {
                        name: stage.name.clone(),
                        after: stage.after.clone(),
                        modes: extract_stage_modes(stage.when.as_ref()),
                    })
                    .collect();
                pipelines.insert(def.name.clone(), stages);
            }
        }
    }
    pipelines
}

/// Extract mode literals from a stage `when` condition.
///
/// Looks for `mode == "literal"` patterns in the expression tree,
/// supporting `||` and `&&` conjunctions.
fn extract_stage_modes(condition: Option<&Expr>) -> BTreeSet<String> {
    let mut modes = BTreeSet::new();
    if let Some(condition) = condition {
        collect_mode_literals(condition, &mut modes);
    }
    modes
}

fn collect_mode_literals(expr: &Expr, modes: &mut BTreeSet<String>) {
    match expr {
        Expr::BinOp(lhs, op, rhs) => match op {
            daglang_syntax::ast::BinOp::Eq => {
                if let Some(mode) = mode_literal_from_equality(lhs, rhs) {
                    modes.insert(mode);
                }
            }
            daglang_syntax::ast::BinOp::And | daglang_syntax::ast::BinOp::Or => {
                collect_mode_literals(lhs, modes);
                collect_mode_literals(rhs, modes);
            }
            _ => {}
        },
        Expr::Guarded(inner, guard) => {
            collect_mode_literals(inner, modes);
            collect_mode_literals(guard, modes);
        }
        Expr::After(inner, _) => collect_mode_literals(inner, modes),
        _ => {}
    }
}

fn mode_literal_from_equality(lhs: &Expr, rhs: &Expr) -> Option<String> {
    let lhs_is_mode = matches!(lhs, Expr::Ident(name) if name == "mode");
    let rhs_is_mode = matches!(rhs, Expr::Ident(name) if name == "mode");
    if lhs_is_mode {
        return literal_string(rhs);
    }
    if rhs_is_mode {
        return literal_string(lhs);
    }
    None
}

fn literal_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(value)) => Some(value.clone()),
        _ => None,
    }
}

/// Extracted pipeline stage info from a parsed `.dag` file.
#[derive(Debug, Clone)]
pub struct PipelineStageInfo {
    pub name: String,
    pub after: Vec<String>,
    /// Mode literals extracted from `when mode == "..."` conditions.
    pub modes: BTreeSet<String>,
}

/// Output from compiling embedded DSL sources.
#[derive(Debug)]
pub struct EmbeddedCompileOutput {
    pub data_values: HashMap<String, serde_json::Value>,
    pub fns: HashMap<String, daglang_lower::LoweredFnBody>,
    /// Pipeline definitions extracted from parsed AST, keyed by pipeline name.
    pub pipelines: HashMap<String, Vec<PipelineStageInfo>>,
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
/// parsing, and module graph construction; this wrapper handles the remaining
/// context-sensitive preparation and then delegates to [`run_compile_pipeline`].
pub fn compile_from_module_graph_with_options(
    context: &DriverContext,
    module_graph: ModuleGraph,
    options: CompileOptions,
) -> Result<CompileOutput, CompileError> {
    let prepared = prepare_compile_context(context, module_graph, &options)?;
    run_compile_pipeline(prepared, options)
}

/// Collect `param` declarations from all modules in the typed project.
pub fn check_from_context(context: &DriverContext) -> Result<CheckOutput, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    check_from_module_graph(module_graph)
}

pub fn check_from_module_graph(module_graph: ModuleGraph) -> Result<CheckOutput, CompileError> {
    let parsed_files = module_graph.modules.len();
    if let Err(errors) = typecheck_module_graph_with_options(
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    ) {
        return Err(CompileError::from(errors));
    }
    Ok(CheckOutput { parsed_files })
}

/// Extract pipeline `param` declarations from a DSL file (permissive mode).
///
/// Parses and typechecks the module graph with `allow_unresolved_imports: true`,
/// then collects all `ParamDecl` items.
/// Lighter weight than `compile_from_context` — no lowering, deriving, or emission.
///
/// **This is a utility path, not mainline compilation.** It uses permissive
/// typechecking to allow partial module graphs where only param declarations
/// are needed.
pub fn load_pipeline_params_permissive(
    context: &DriverContext,
) -> Result<Vec<PipelineParam>, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    let typed = typecheck_module_graph_with_options(
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .map_err(CompileError::from)?;
    Ok(typed.pipeline_params().to_vec())
}

/// Generate Rust type definitions from DSL TypeDefs (permissive mode).
///
/// Typechecks the module graph with `allow_unresolved_imports: true`, then
/// extracts all `TypeDef` items and converts them to Rust struct/enum
/// definitions via `type_codegen`.
///
/// **This is a utility path, not mainline compilation.** Type generation only
/// needs structural defs, not service bindings, so permissive typechecking
/// is acceptable.
pub fn generate_types_from_context_permissive(
    context: &DriverContext,
    module_filter: &[&str],
) -> Result<String, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    // Type generation only needs structural defs, not service bindings.
    let typed = typecheck_module_graph_with_options(
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: true,
        },
    )
    .map_err(CompileError::from)?;
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
        &module_graph,
        TypecheckOptions {
            allow_unresolved_imports: false,
        },
    )
    .map_err(CompileError::from)?;
    Ok(lint_report_coverage(&typed))
}

fn lint_report_coverage(typed: &TypedProject<'_>) -> Vec<ReportCoverageIssue> {
    let mut issues = Vec::new();

    for module in typed.modules() {
        let module_name = module.module_path.as_dotted();
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
                Stmt::Node(ns) => {
                    by_binding.insert(ns.name.clone(), stage.name.clone());
                }
                Stmt::Expr(_) | Stmt::Return(_) => {}
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
        Stmt::Node(ns) => {
            collect_covered_stages_from_expr(&ns.expr, producer_by_binding, covered);
        }
        Stmt::Return(fields) => {
            for (_name, expr) in fields {
                collect_covered_stages_from_expr(expr, producer_by_binding, covered);
            }
        }
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
            match body {
                daglang_syntax::ast::ForBody::Expr(expr) => {
                    collect_covered_stages_from_expr(expr, producer_by_binding, covered);
                }
                daglang_syntax::ast::ForBody::Block(stmts) => {
                    for stmt in stmts {
                        match stmt {
                            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                                collect_covered_stages_from_expr(
                                    expr,
                                    producer_by_binding,
                                    covered,
                                );
                            }
                            Stmt::Node(node_stmt) => {
                                collect_covered_stages_from_expr(
                                    &node_stmt.expr,
                                    producer_by_binding,
                                    covered,
                                );
                                if let Some(guard) = &node_stmt.when_guard {
                                    collect_covered_stages_from_expr(
                                        guard,
                                        producer_by_binding,
                                        covered,
                                    );
                                }
                            }
                            Stmt::Return(fields) => {
                                for (_, expr) in fields {
                                    collect_covered_stages_from_expr(
                                        expr,
                                        producer_by_binding,
                                        covered,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Expr::Pipe(lhs, rhs) => {
            collect_covered_stages_from_expr(lhs, producer_by_binding, covered);
            collect_covered_stages_from_expr(rhs, producer_by_binding, covered);
        }
        Expr::PipeCall(receiver, _, args) => {
            collect_covered_stages_from_expr(receiver, producer_by_binding, covered);
            for (_name, arg_expr) in args {
                collect_covered_stages_from_expr(arg_expr, producer_by_binding, covered);
            }
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
            match body {
                daglang_syntax::ast::ForBody::Expr(expr) => collect_root_identifiers(expr, roots),
                daglang_syntax::ast::ForBody::Block(stmts) => {
                    for stmt in stmts {
                        match stmt {
                            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                                collect_root_identifiers(expr, roots);
                            }
                            Stmt::Node(node_stmt) => {
                                collect_root_identifiers(&node_stmt.expr, roots);
                                if let Some(guard) = &node_stmt.when_guard {
                                    collect_root_identifiers(guard, roots);
                                }
                            }
                            Stmt::Return(fields) => {
                                for (_, expr) in fields {
                                    collect_root_identifiers(expr, roots);
                                }
                            }
                        }
                    }
                }
            }
        }
        Expr::Pipe(lhs, rhs) => {
            collect_root_identifiers(lhs, roots);
            collect_root_identifiers(rhs, roots);
        }
        Expr::PipeCall(receiver, _, args) => {
            collect_root_identifiers(receiver, roots);
            for (_name, arg_expr) in args {
                collect_root_identifiers(arg_expr, roots);
            }
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

/// Cached discovery metadata for incremental compilation (C26).
///
/// Stores the source digest and extracted metadata from a compilation.
/// When the source digest matches on a subsequent run, the cached metadata
/// is returned without recompilation (including func params, so no re-parse needed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDiscoveryEntry {
    /// SHA-256 of sorted source file content hashes.
    pub source_digest: String,
    /// Inferred entrypoints from compilation.
    pub entrypoints: Vec<CachedEntrypoint>,
    /// Output paths extracted from the DAG.
    pub output_paths: Vec<String>,
    /// Cached func parameters — avoids re-parsing the AST on cache hit.
    #[serde(default)]
    pub func_params: BTreeMap<String, Vec<CachedFuncParam>>,
}

/// Serializable entrypoint metadata for caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntrypoint {
    pub func_name: String,
    pub module: String,
    pub node_id: String,
}

/// Serializable func parameter metadata for caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFuncParam {
    pub name: String,
    pub type_name: String,
    pub cardinality: String,
    pub default: Option<String>,
}

impl From<&InferredEntrypoint> for CachedEntrypoint {
    fn from(ep: &InferredEntrypoint) -> Self {
        CachedEntrypoint {
            func_name: ep.func_name.clone(),
            module: ep.module.clone(),
            node_id: ep.node_id.clone(),
        }
    }
}

impl From<&CachedEntrypoint> for InferredEntrypoint {
    fn from(cached: &CachedEntrypoint) -> Self {
        InferredEntrypoint {
            func_name: cached.func_name.clone(),
            module: cached.module.clone(),
            node_id: cached.node_id.clone(),
        }
    }
}

fn discover_module_graph_for_context(context: &DriverContext) -> Result<ModuleGraph, CompileError> {
    if let Some(target_file) = &context.target_file {
        return discover_target_module_graph_for_context(context, target_file);
    }

    ModuleGraph::discover_strict(&context.roots).map_err(CompileError::Resolve)
}

fn discover_target_module_graph_for_context(
    context: &DriverContext,
    target_file: &Path,
) -> Result<ModuleGraph, CompileError> {
    let canonical_roots = daglang_resolve::canonicalize_roots(&context.roots);
    let mut modules: Vec<ResolvedModule> = Vec::new();
    let mut imports_by_index: Vec<Vec<ModulePath>> = Vec::new();
    let mut module_index_by_path: HashMap<PathBuf, usize> = HashMap::new();
    let mut module_index_by_decl: HashMap<ModulePath, usize> = HashMap::new();

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
        let imports = imports_by_index.get(module_index).cloned().ok_or_else(|| {
            CompileError::Message(format!(
                "internal module graph error: missing import list for module index {module_index}"
            ))
        })?;
        let mut dependencies = Vec::new();
        for import in imports {
            if let Some(dep_index) = module_index_by_decl.get(&import).copied() {
                dependencies.push(dep_index);
                continue;
            }
            let import_file = resolve_import_file_path(&context.roots, &import)?;
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
    active_profile: Option<&str>,
) -> Result<(), CompileError> {
    let mut seed_files = Vec::<PathBuf>::new();
    for root in roots {
        let discovery_root = root.join("profiles");
        if !discovery_root.is_dir() {
            continue;
        }
        let mut discovered =
            daglang_resolve::discover_dag_files(&discovery_root).map_err(CompileError::Resolve)?;
        seed_files.append(&mut discovered);
    }
    seed_files.sort();
    seed_files.dedup();
    if seed_files.is_empty() {
        return Ok(());
    }

    if active_profile.is_some() {
        let canonical_roots = daglang_resolve::canonicalize_roots(roots);
        let mut implementation_modules = Vec::<PathBuf>::new();
        for profile_file in &seed_files {
            let (module, _imports) =
                parse_target_module_file(profile_file, roots, &canonical_roots)?;
            for item in &module.ast.items {
                let Item::ProfileDef(def) = &item.node else {
                    continue;
                };
                // Only include implementation modules for the active profile,
                // not all profiles. This avoids loading transport-incomplete
                // providers that aren't used by the active profile.
                if active_profile != Some(def.name.as_str()) {
                    continue;
                }
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
                return Err(CompileError::Resolve(ResolveError::DuplicateModule(
                    module.module_path,
                )));
            }
        } else if let Some(existing_index) = module_index_by_decl.get(&module.module_path).copied()
        {
            let existing = &module_graph.modules[existing_index];
            if existing.path != module.path {
                return Err(CompileError::Resolve(ResolveError::DuplicateModule(
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
            let import_file = resolve_import_file_path(roots, &import)?;
            pending.push_back(import_file);
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
            .filter_map(|import| module_index_by_decl.get(&import.node.path).copied())
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
    let segments: Vec<String> = implementation_type
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect();
    if segments.len() < 2 {
        return None;
    }
    for end in (1..segments.len()).rev() {
        let prefix = ModulePath::new(segments[..end].to_vec());
        if let Ok(path) = resolve_import_file_path(roots, &prefix) {
            return Some(path);
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn add_target_module_if_applicable(
    path: &Path,
    expected_module_path: Option<&ModulePath>,
    roots: &[PathBuf],
    canonical_roots: &[PathBuf],
    modules: &mut Vec<ResolvedModule>,
    imports_by_index: &mut Vec<Vec<ModulePath>>,
    module_index_by_path: &mut HashMap<PathBuf, usize>,
    module_index_by_decl: &mut HashMap<ModulePath, usize>,
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
        if &module.module_path != expected {
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
            return Err(CompileError::Resolve(ResolveError::DuplicateModule(
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
) -> Result<(ResolvedModule, Vec<ModulePath>), CompileError> {
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
        CompileError::Resolve(ResolveError::ParseErrors(vec![(
            path.to_path_buf(),
            diagnostics,
        )]))
    })?;
    let module_path = ast
        .module_path
        .as_ref()
        .map(|module| module.node.clone())
        .map_or_else(
            || {
                daglang_resolve::path_to_module_path(path, roots, canonical_roots)
                    .map_err(CompileError::Resolve)
            },
            Ok,
        )?;
    let imports: Vec<ModulePath> = ast
        .imports
        .iter()
        .map(|import| import.node.path.clone())
        .collect();
    Ok((
        ResolvedModule {
            path: path.to_path_buf(),
            ast,
            module_path,
            dependencies: Vec::new(),
            source,
        },
        imports,
    ))
}

fn resolve_import_file_path(
    roots: &[PathBuf],
    import_path: &ModulePath,
) -> Result<PathBuf, CompileError> {
    let mut relative = PathBuf::new();
    for segment in &import_path.segments {
        relative.push(segment);
    }
    relative.set_extension("dag");
    roots
        .iter()
        .map(|root| root.join(&relative))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            CompileError::Message(format!(
                "unresolved import: {}",
                import_path.as_dotted()
            ))
        })
}

fn callable_scope_for_context(
    context: &DriverContext,
    module_graph: &ModuleGraph,
) -> Result<Option<(HashSet<String>, String)>, CompileError> {
    let Some(target_index) = target_module_index_for_context(context, module_graph)? else {
        return Ok(None);
    };
    let Some(target_module) = module_graph.modules.get(target_index) else {
        return Err("internal error: target module index out of bounds".into());
    };
    let has_callable_items = module_has_callable_items(target_module);
    if !has_callable_items {
        return Ok(None);
    }
    let entry_module_name = target_module.module_path.as_dotted();
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
        scope.insert(module.module_path.as_dotted());
        for dependency in &module.dependencies {
            queue.push_back(*dependency);
        }
    }
    if scope.is_empty() {
        scope.insert(entry_module_name.clone());
    }
    Ok(Some((scope, entry_module_name)))
}

fn target_module_name_for_context(
    context: &DriverContext,
    module_graph: &ModuleGraph,
) -> Result<Option<String>, CompileError> {
    let Some(target_index) = target_module_index_for_context(context, module_graph)? else {
        return Ok(None);
    };
    Ok(module_graph
        .modules
        .get(target_index)
        .map(|module| module.module_path.as_dotted()))
}

fn target_module_index_for_context(
    context: &DriverContext,
    module_graph: &ModuleGraph,
) -> Result<Option<usize>, CompileError> {
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
    Ok(Some(target_index))
}

fn module_has_callable_items(module: &ResolvedModule) -> bool {
    module
        .ast
        .items
        .iter()
        .any(|item| item.node.as_callable().is_some() || matches!(item.node, Item::PipelineDef(_)))
}

/// Collect `fn` items whose body was lossy-parsed (parser recovery).
///
/// A lossy `fn` body means the parser could not fully capture the pure
/// function body. `run_compile_pipeline` rejects any compilation where
/// this list is non-empty — lossy passthroughs violate README invariant §7
/// ("every expression lowers to structural DAG nodes or the compilation
/// fails").
///
/// Returns a list of `"module::name"` strings for all affected `fn` items.
fn collect_lossy_fn_bodies(graph: &ModuleGraph) -> Vec<String> {
    let mut lossy_items = Vec::new();
    for module in &graph.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            if let Item::FnDef(def) = &item.node {
                if def.body.lossy {
                    lossy_items.push(format!("{}::{}", module_name, def.name));
                }
            }
        }
    }
    lossy_items
}

/// Merge two sorted path lists into one sorted, deduplicated list.
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
            let declared = module.module_path.as_dotted();
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
    config_resolver: impl Fn(&ModulePath, &Path) -> Option<daglang_emit::test_mock_emit::TestEmitConfig>,
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
                module.module_path.as_dotted()
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
    use gunbc_test::unique_temp_dir;
    use std::collections::HashSet;

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
        let error_text = error.to_string();
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
            error.to_string().contains("unresolved import"),
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
            error.to_string().contains("unresolved call target"),
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
    transport rest { method: GET, path: "/ok" }
  }
}"#,
        )
        .expect("failed to write service source");
        std::fs::write(
            root.join("profiles/local.dag"),
            r#"module profiles.local
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
    transport rest { method: GET, path: "/ok" }
  }
}"#,
        )
        .expect("failed to write service source");
        std::fs::write(
            root.join("profiles/local.dag"),
            r#"module profiles.local
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
                edge.to_node.0 == "pipelines.main::run"
                    && edge.to_port.0 == gunbc_ir::types::PortName::DEPS
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
    fn run_compile_pipeline_from_prepared_context_preserves_receipt_digest() {
        let root = unique_temp_dir("compile_prepared_context");
        std::fs::create_dir_all(&root).expect("failed to create temp root");
        let file = root.join("sample.dag");
        std::fs::write(&file, "module sample\nfn run() -> Unit {}\n")
            .expect("failed to write source");

        let context = DriverContext {
            roots: vec![root.clone()],
            target_file: Some(file.clone()),
        };
        let module_graph =
            discover_module_graph_for_context(&context).expect("module graph should discover");
        let options = CompileOptions::default();
        let prepared = prepare_compile_context(&context, module_graph, &options)
            .expect("prepared context should succeed");
        let output = run_compile_pipeline(prepared, options)
            .expect("stage runner should compile prepared context");

        assert!(
            output
                .lowered_dag
                .nodes
                .iter()
                .any(|node| node.id.0 == "sample::run"),
            "compiled DAG should include the target callable"
        );
        let receipt = output
            .receipt
            .expect("prepared compilation should emit a receipt");
        let expected_digest =
            compute_source_digest(&[file]).expect("legacy path-based digest should succeed");
        assert_eq!(
            receipt.source_digest, expected_digest,
            "prepared pipeline digest should match the legacy path-based digest"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    // DELETED: compile_with_exec_runtime_layer_emits_exec_runtime_bundle
    // Blocked on: RF-E5 (PureRender fn body delegate gap — exec-runtime can't classify Callable with fn_body).
    // Restore when exec-runtime gains fn body classification support.

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
                .to_string()
                .contains("layer 1 currently supports only --target rust"),
            "expected unsupported target error, got: {error}"
        );

        std::fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn compile_with_go_native_layer_emits_go_bundle() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/readme.dag");
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
        let file = root.join("tools/readme.dag");
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

        assert_eq!(
            output.emit_manifest_path,
            "target/generated/go/emit_manifest.json"
        );
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

    // DELETED: makegen_exec_runtime_e2e_structural_verification
    // Tracked as RF-E6 in tasks.md — exec-runtime emit missing LoadRegistry
    // handler + PureRender fn classification. Re-add after exec-runtime emitter
    // handles all makegen node classifications.

    // DELETED: pragma_exec_runtime_e2e_structural_verification
    // Tracked as RF-E6 in tasks.md — exec-runtime emit missing
    // ContentUpsertOutputPath classification + PureRender fn handlers.
    // Re-add after exec-runtime emitter handles all pragma node classifications.

    // ---- Determinism contract tests ----

    /// Helper: compile a target and return the receipt, panicking on failure.
    fn compile_with_receipt(root: &Path, target_file: Option<&Path>) -> CompileReceipt {
        let context = DriverContext {
            roots: vec![root.to_path_buf()],
            target_file: target_file.map(|p| p.to_path_buf()),
        };
        let output =
            compile_from_context(&context).expect("compile should succeed for determinism test");
        output.receipt.expect("receipt should be present")
    }

    #[test]
    fn determinism_single_file_compile_produces_identical_receipts() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let root = workspace.join("dsl");
        let file = root.join("tools/readme.dag");

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

    #[test]
    fn verify_dag_catches_unwired_inputs_when_enabled() {
        // Build a minimal DAG with a deliberately unwired required input,
        // then confirm verify_dag surfaces it.
        use gunbc_ir::validate::{validate_required_inputs, verify_dag};
        use gunbc_ir::{build::port, Dag, Edge, Node};

        let mut dag: Dag<()> = Dag::new();
        dag.add_node(Node::opaque(
            "source",
            vec![],
            vec![port("out", "String")],
            (),
        ));
        dag.add_node(Node::opaque(
            "sink",
            vec![port("data", "String"), port("config", "String")],
            vec![port("result", "String")],
            (),
        ));
        // Wire only "data", leave "config" unwired
        dag.add_edge(Edge::new("source", "out", "sink", "data"));

        let errors = validate_required_inputs(&dag);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].port_name, "config");

        let all_errors = verify_dag(&dag);
        assert!(
            all_errors.iter().any(|e| matches!(
                e,
                gunbc_ir::VerifyError::UnwiredInput(err) if err.port_name == "config"
            )),
            "verify_dag should detect unwired 'config' input"
        );
    }

    #[test]
    fn structural_primitive_validation_flags_unwired_required_input() {
        use daglang_lower::{PrimitiveLiteral, PrimitiveOpKind};
        use gunbc_ir::{build::port, Dag, Edge, Node, VerifyError};

        let mut dag: Dag<LoweredOp> = Dag::new();
        dag.add_node(Node::opaque(
            "left_src",
            vec![],
            vec![port("out", "Any")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "left_src".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::Int(1),
                },
            },
        ));
        dag.add_node(Node::opaque(
            "binary",
            vec![port("left", "Any"), port("right", "Any")],
            vec![port("result", "Any")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "binary".to_string(),
                kind: PrimitiveOpKind::BinaryOp {
                    op: daglang_lower::expr::LoweredBinOp::Add,
                },
            },
        ));
        dag.add_edge(Edge::new("left_src", "out", "binary", "left"));

        let errors = crate::pipeline::validate_structural_primitive_input_wiring(&dag);
        assert!(
            errors.iter().any(|error| matches!(
                error,
                VerifyError::UnwiredInput(unwired)
                    if unwired.node_id == "binary" && unwired.port_name == "right"
            )),
            "expected unwired required input error for binary.right, got: {errors:?}"
        );
    }

    #[test]
    fn structural_primitive_validation_allows_conditional_without_else_input_port() {
        use daglang_lower::{PrimitiveLiteral, PrimitiveOpKind};
        use gunbc_ir::{build::port, Dag, Edge, Node};

        let mut dag: Dag<LoweredOp> = Dag::new();
        dag.add_node(Node::opaque(
            "cond_src",
            vec![],
            vec![port("out", "Bool")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "cond_src".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::Bool(true),
                },
            },
        ));
        dag.add_node(Node::opaque(
            "then_src",
            vec![],
            vec![port("out", "Any")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "then_src".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::String("ok".to_string()),
                },
            },
        ));
        dag.add_node(Node::opaque(
            "conditional",
            vec![port("condition", "Bool"), port("then", "Any")],
            vec![port("result", "Any")],
            LoweredOp::Primitive {
                module: "test".to_string(),
                name: "conditional".to_string(),
                kind: PrimitiveOpKind::Conditional,
            },
        ));
        dag.add_edge(Edge::new("cond_src", "out", "conditional", "condition"));
        dag.add_edge(Edge::new("then_src", "out", "conditional", "then"));

        let errors = crate::pipeline::validate_structural_primitive_input_wiring(&dag);
        assert!(
            errors.is_empty(),
            "conditional without else input port should pass, got: {errors:?}"
        );
    }

    #[test]
    fn compile_data_from_module_returns_same_data_as_sources() {
        // Parity test: compile_data_from_module_permissive (filesystem) should produce the
        // same data_values as compile_data_from_sources_permissive (include_str).
        let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout");
        let dsl_root = layout.workspace_root.join("dsl");

        let module_output =
            compile_data_from_module_permissive(&dsl_root, "config/codegen_paths.dag")
                .expect("compile_data_from_module should succeed");

        // Verify it found expected data keys from the module
        assert!(
            !module_output.data_values.is_empty(),
            "config/codegen_paths.dag should produce data values"
        );
        assert!(
            module_output.data_values.contains_key("bin_dir"),
            "config/codegen_paths.dag should declare `bin_dir`, got keys: {:?}",
            module_output.data_values.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn compile_data_from_module_resolves_transitive_imports() {
        // readme.dag has transitive imports (std/*, extdeps/*).
        // compile_data_from_module should resolve them all automatically.
        let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout");
        let dsl_root = layout.workspace_root.join("dsl");

        let output = compile_data_from_module_permissive(&dsl_root, "tools/readme.dag")
            .expect("compile_data_from_module should resolve transitive imports");

        // readme.dag has fn items, not just data declarations
        assert!(
            !output.fns.is_empty(),
            "tools/readme.dag should have extractable fn bodies"
        );
    }
}
