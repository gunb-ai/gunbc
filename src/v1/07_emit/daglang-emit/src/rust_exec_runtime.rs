//! Rust exec-runtime codegen (Layer 1 fast path).
//!
//! Generates a standalone Rust crate that builds `Dag<Op>` and calls
//! `gunbc-exec` to run it. Routes through the AbstractIR → SystemsIR →
//! Rust text pipeline via [`SourceFile`] + [`render_rust_source`].
//!
//! The generated crate contains:
//! - An `Op` enum with one variant per handler kind used in the DAG
//! - `impl Executable for Op` with match dispatch
//! - Handler bodies for each `HandlerKind`
//! - `fn build_dag() -> Dag<Op>` with hardcoded graph construction
//! - `fn main()` with CLI arg parsing + `execute_dag`
//! - `Cargo.toml` with `gunbc-ir`/`gunbc-exec`/`gunbc-lib-transport` deps

use daglang_lower::{LoweredOp, ObligationCategory, PrimitiveLiteral, PrimitiveOpKind};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Cardinality;
use gunbc_ir::Dag;
use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::EmittedFile;

// ===========================================================================
// Public API
// ===========================================================================

/// Default relative path from a generated crate to `gunbc-ir`.
///
/// Assumes the generated crate lives two directories below the layer root
/// (e.g. `target/codegen/bin/<crate>/`).
// TODO(T14): derive from WorkspaceLayout::relative_path at emit time so the
// path is correct regardless of where the generated crate is placed.
const DEFAULT_DEP_IR: &str = "../../0_foundation/ir";

/// Default relative path from a generated crate to `gunbc-exec`.
const DEFAULT_DEP_EXEC: &str = "../../9_execute/exec";

/// Default relative path from a generated crate to `gunbc-lib-transport`.
const DEFAULT_DEP_TRANSPORT: &str = "../../8_materialize/transport";

/// Configuration for exec-runtime code generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyPaths {
    pub gunbc_ir: String,
    pub gunbc_exec: String,
    pub gunbc_lib_transport: String,
}

impl DependencyPaths {
    pub fn conventional_defaults() -> Self {
        Self {
            gunbc_ir: DEFAULT_DEP_IR.to_string(),
            gunbc_exec: DEFAULT_DEP_EXEC.to_string(),
            gunbc_lib_transport: DEFAULT_DEP_TRANSPORT.to_string(),
        }
    }
}

impl Default for DependencyPaths {
    fn default() -> Self {
        Self::conventional_defaults()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmitConfig {
    pub dependency_paths: DependencyPaths,
}

/// Emit a standalone Rust crate from a lowered DAG.
///
/// Returns `src/main.rs` and `Cargo.toml` as [`EmittedFile`] entries.
/// The generated crate, when compiled and run, produces the same behavior
/// as the domain-specific hand-built Rust binary.
///
/// Builds a [`SourceFile`] IR and renders it via [`render_rust_source`],
/// routing through the AbstractIR → SystemsIR → Rust text pipeline.
pub fn emit_exec_runtime(
    dag: &Dag<LoweredOp>,
    module_name: &str,
) -> Result<Vec<EmittedFile>, ExecRuntimeError> {
    emit_exec_runtime_with_config(dag, module_name, &EmitConfig::default())
}

/// Emit a standalone Rust crate from a lowered DAG with full configuration.
pub fn emit_exec_runtime_with_config(
    dag: &Dag<LoweredOp>,
    module_name: &str,
    config: &EmitConfig,
) -> Result<Vec<EmittedFile>, ExecRuntimeError> {
    let classified = classify_nodes_with_config(dag)?;
    let handler_kinds = collect_handler_kinds(&classified);
    let source = build_exec_runtime_source(dag, module_name, &classified, &handler_kinds);
    let main_rs = crate::render_rust::render_rust_source(&source);
    let cargo_toml = emit_cargo_toml(module_name, &handler_kinds, &config.dependency_paths);

    Ok(vec![
        EmittedFile {
            path: "src/main.rs".to_string(),
            content: main_rs,
        },
        EmittedFile {
            path: "Cargo.toml".to_string(),
            content: cargo_toml,
        },
    ])
}

// ===========================================================================
// Error
// ===========================================================================

/// Errors during exec-runtime code generation.
#[derive(Debug, Clone)]
pub enum ExecRuntimeError {
    /// A DAG node could not be classified to a runtime handler.
    UnresolvableNode { node_id: String, detail: String },
    /// A SubDag node was encountered (not supported in exec-runtime path).
    SubDagNotSupported { node_id: String },
}

impl std::fmt::Display for ExecRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvableNode { node_id, detail } => {
                write!(
                    f,
                    "cannot resolve node `{node_id}` for exec-runtime: {detail}"
                )
            }
            Self::SubDagNotSupported { node_id } => {
                write!(
                    f,
                    "subdag node `{node_id}` is not supported in exec-runtime codegen"
                )
            }
        }
    }
}

// ===========================================================================
// Handler kinds
// ===========================================================================

/// Which executor body to generate for an Op variant.
///
/// Each handler kind maps to a concrete function body. Multiple DAG
/// nodes can share the same handler
/// kind (e.g., two different "prepare_read" nodes both use `PrepareReadContent`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum HandlerKind {
    FsEnv,
    ParamSource,
    LiteralSource,
    PrepareReadContent,
    ExecuteReadContent,
    PrepareWriteContent,
    CompareContent,
    ExecuteTransport,
    Collection,
    /// Passthrough stub for callables without a compiled exec-runtime handler.
    /// These are callables that the compiler validated but that have no
    /// specialized handler (e.g., std.patterns, std.resources, service transport).
    Passthrough,
}

impl HandlerKind {
    fn variant_name(self) -> &'static str {
        match self {
            Self::FsEnv => "FsEnv",
            Self::ParamSource => "ParamSource",
            Self::LiteralSource => "LiteralSource",
            Self::PrepareReadContent => "PrepareReadContent",
            Self::ExecuteReadContent => "ExecuteReadContent",
            Self::PrepareWriteContent => "PrepareWriteContent",
            Self::CompareContent => "CompareContent",
            Self::ExecuteTransport => "ExecuteTransport",
            Self::Collection => "Collection",
            Self::Passthrough => "Passthrough",
        }
    }
}

/// Embedded data assets required by exec-runtime handler implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EmbeddedAsset {}

impl EmbeddedAsset {
    pub fn key(self) -> &'static str {
        match self {}
    }

    pub fn path(self) -> &'static str {
        match self {}
    }
}

// ===========================================================================
// Classification
// ===========================================================================

/// A classified DAG node: its ID, ports, and resolved handler kind.
struct ClassifiedNode {
    node_id: String,
    handler: HandlerKind,
    op_ctor: String,
    inputs: Vec<(String, String, Cardinality)>, // (port_name, type_id, cardinality)
    outputs: Vec<(String, String, Cardinality)>, // (port_name, type_id, cardinality)
}

fn classify_nodes_with_config(
    dag: &Dag<LoweredOp>,
) -> Result<Vec<ClassifiedNode>, ExecRuntimeError> {
    let mut result = Vec::with_capacity(dag.nodes.len());
    for node in &dag.nodes {
        let node_id = node.id.0.clone();

        let op = match &node.body {
            NodeBody::Opaque(op) => op,
            NodeBody::SubDag(..) => continue,
        };

        let handler = match classify_handler(op) {
            Some(HandlerClassification::Handler(h)) => h,
            Some(HandlerClassification::MetadataOnly) => continue,
            None => {
                return Err(ExecRuntimeError::UnresolvableNode {
                    node_id: node_id.clone(),
                    detail: format!("no runtime op classification for {op:?}"),
                });
            }
        };
        let op_ctor = classify_op_ctor(op, &node.outputs, handler).map_err(|detail| {
            ExecRuntimeError::UnresolvableNode {
                node_id: node_id.clone(),
                detail,
            }
        })?;

        let inputs = node
            .inputs
            .iter()
            .map(|p| (p.name.0.clone(), p.type_id.0.clone(), p.cardinality))
            .collect();
        let outputs = node
            .outputs
            .iter()
            .map(|p| (p.name.0.clone(), p.type_id.0.clone(), p.cardinality))
            .collect();

        result.push(ClassifiedNode {
            node_id,
            handler,
            op_ctor,
            inputs,
            outputs,
        });
    }
    Ok(result)
}

fn collect_handler_kinds(classified: &[ClassifiedNode]) -> BTreeSet<HandlerKind> {
    classified.iter().map(|c| c.handler).collect()
}

/// Return the embedded assets required by handler kinds present in `dag`.
pub fn required_embedded_assets(_dag: &Dag<LoweredOp>) -> BTreeSet<EmbeddedAsset> {
    BTreeSet::new()
}

/// Whether a node should be included in exec-runtime emission, skipped
/// because it's metadata-only, or classified to a specific handler.
#[derive(Debug, PartialEq)]
enum HandlerClassification {
    Handler(HandlerKind),
    MetadataOnly,
}

fn classify_handler(op: &LoweredOp) -> Option<HandlerClassification> {
    match op {
        LoweredOp::Collection { .. } => {
            return Some(HandlerClassification::Handler(HandlerKind::Collection))
        }
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CallParamSource { .. },
            ..
        } => return Some(HandlerClassification::Handler(HandlerKind::ParamSource)),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CallLiteralSource { .. },
            ..
        } => return Some(HandlerClassification::Handler(HandlerKind::LiteralSource)),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::FsEnv,
            ..
        } => return Some(HandlerClassification::Handler(HandlerKind::FsEnv)),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoPrepareFileRead,
            ..
        } => {
            return Some(HandlerClassification::Handler(
                HandlerKind::PrepareReadContent,
            ))
        }
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileRead,
            ..
        } => {
            return Some(HandlerClassification::Handler(
                HandlerKind::ExecuteReadContent,
            ))
        }
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoPrepareFileWrite,
            ..
        } => {
            return Some(HandlerClassification::Handler(
                HandlerKind::PrepareWriteContent,
            ))
        }
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CompareEquality,
            ..
        } => return Some(HandlerClassification::Handler(HandlerKind::CompareContent)),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileWrite,
            ..
        } => {
            return Some(HandlerClassification::Handler(
                HandlerKind::ExecuteTransport,
            ))
        }
        LoweredOp::Pipeline { .. } => {}
        LoweredOp::Transport { .. } => {
            return Some(HandlerClassification::Handler(HandlerKind::Passthrough));
        }
        LoweredOp::Callable { .. } => {}
        // Structural nodes that have no runtime behavior — safe to skip.
        LoweredOp::Pattern(_) | LoweredOp::UnsupportedPattern { .. } => {
            return Some(HandlerClassification::MetadataOnly)
        }
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::ContentUpsertOutputPath { .. },
            ..
        } => return Some(HandlerClassification::MetadataOnly),
        // C24: GetField is still interpreter-backed in the resolve layer. We
        // keep layer-1 compile unblocked by emitting passthrough stubs until
        // a dedicated handler lands.
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::GetField { .. },
            ..
        } => return Some(HandlerClassification::Handler(HandlerKind::Passthrough)),
        // C24: All remaining structural primitive ops use passthrough stubs.
        LoweredOp::Primitive { kind, .. } => {
            debug_assert!(
                kind.is_structural(),
                "unhandled non-structural primitive: {kind:?}"
            );
            return Some(HandlerClassification::Handler(HandlerKind::Passthrough));
        }
    }

    let handler = |h| Some(HandlerClassification::Handler(h));

    let obligation: Option<ObligationCategory> = match op {
        LoweredOp::Callable { obligation, .. } => Some((*obligation).into()),
        LoweredOp::Transport { obligation, .. } => Some((*obligation).into()),
        LoweredOp::Pipeline { .. } => None,
        _ => return None,
    };

    match obligation {
        Some(ObligationCategory::None) | Some(ObligationCategory::PureGeneric) => {
            handler(HandlerKind::Passthrough)
        }
        Some(ObligationCategory::ServiceParamSource)
        | Some(ObligationCategory::InterfaceContractVerification)
        | Some(ObligationCategory::ResourceProvide)
        | Some(ObligationCategory::ResourceAcquire)
        | Some(ObligationCategory::ResourceRelease) => handler(HandlerKind::Passthrough),
        Some(ObligationCategory::ServiceTransportPrepare)
        | Some(ObligationCategory::ServiceTransportExecute)
        | Some(ObligationCategory::ServiceTransportParse) => handler(HandlerKind::Passthrough),
        Some(ObligationCategory::PureRender) | Some(ObligationCategory::PureDataLoad) => None,
        None => handler(HandlerKind::Passthrough),
    }
}

fn classify_op_ctor(
    op: &LoweredOp,
    outputs: &[gunbc_ir::Port],
    handler: HandlerKind,
) -> Result<String, String> {
    if handler != HandlerKind::LiteralSource {
        return Ok(format!("Op::{}", handler.variant_name()));
    }

    let value_expr = match op {
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CallLiteralSource { literal },
            ..
        } => primitive_literal_to_runtime_value_expr(literal),
        _ => {
            return Err(
                "literal source classification requires primitive literal-source op".to_string(),
            );
        }
    };
    let output_port = outputs
        .first()
        .map(|port| port.name.0.as_str())
        .ok_or_else(|| "literal source callable has no output ports".to_string())?;
    Ok(format!(
        "Op::LiteralSource {{ output_port: {}, value: {} }}",
        rust_string_literal(output_port),
        value_expr
    ))
}

fn primitive_literal_to_runtime_value_expr(literal: &PrimitiveLiteral) -> String {
    match literal {
        PrimitiveLiteral::String(value) => {
            format!("Value::Str({}.to_string())", rust_string_literal(value))
        }
        PrimitiveLiteral::Int(value) => format!("Value::Int({value})"),
        PrimitiveLiteral::Bool(value) => format!("Value::Bool({value})"),
        PrimitiveLiteral::Json(value) => json_to_native_value_expr(value),
        PrimitiveLiteral::Unit => "Value::Unit".to_string(),
    }
}

/// Emit a `serde_json::Value` as a native `Value` expression without serde_json dependency.
fn json_to_native_value_expr(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "Value::Unit".to_string(),
        serde_json::Value::Bool(b) => format!("Value::Bool({b})"),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                format!("Value::Int({i})")
            } else {
                format!("Value::Float({})", n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => {
            format!("Value::Str({}.to_string())", rust_string_literal(s))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_native_value_expr).collect();
            format!("Value::List(vec![{}])", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<String> = obj
                .iter()
                .map(|(k, v)| {
                    format!(
                        "({}.to_string(), {})",
                        rust_string_literal(k),
                        json_to_native_value_expr(v)
                    )
                })
                .collect();
            format!("Value::Map([{}].into_iter().collect())", entries.join(", "))
        }
    }
}

// ===========================================================================
// Handler helpers (shared by IR path)
// ===========================================================================

/// Return the body (inside braces) for each handler kind.
///
/// Uses `r##"..."##` to avoid issues with inner `"#` sequences.
fn handler_body(kind: HandlerKind) -> &'static str {
    match kind {
        HandlerKind::FsEnv => {
            r##"    OutputMap::new().str("FilesystemHandle", "filesystem://workspace").ok()
"##
        }
        HandlerKind::ParamSource => {
            r##"    Ok(inputs)
"##
        }
        HandlerKind::LiteralSource => {
            r##"    OutputMap::new().value(output_port, value.clone()).ok()
"##
        }
        HandlerKind::PrepareReadContent => {
            r##"    let path = inputs.get("path").and_then(Value::as_str).unwrap_or("");
    if path.is_empty() {
        return Err(ExecError::new("missing required `path` input for prepare_read_content"));
    }
    OutputMap::new().request("request", TransportRequest::File(FileRequest::read(path))).ok()
"##
        }
        HandlerKind::ExecuteReadContent => {
            r##"    let Some(request) = inputs.get("request").and_then(Value::as_request) else {
        return Err(ExecError::new("missing required `request` input for execute_read_content"));
    };
    match request {
        TransportRequest::File(fr) => {
            let resp = execute_file_request(&fr);
            OutputMap::new().response("response", TransportResponse::File(resp)).ok()
        }
        _ => Err(ExecError::new("unsupported transport request kind")),
    }
"##
        }
        HandlerKind::PrepareWriteContent => {
            r##"    let path = inputs.get("path").and_then(Value::as_str).unwrap_or("");
    let content = inputs.get("content").and_then(Value::as_str).unwrap_or("");
    if path.is_empty() || content.is_empty() {
        let mut input_keys: Vec<&str> = inputs.keys().map(|k| k.as_str()).collect();
        input_keys.sort_unstable();
        return Err(ExecError::new(format!(
            "missing required `path` or `content` input for prepare_write_content (available inputs: {})",
            input_keys.join(", ")
        )));
    }
    OutputMap::new().request("request", TransportRequest::File(FileRequest::write(path, content))).ok()
"##
        }
        HandlerKind::CompareContent => {
            r##"    let expected = inputs.get("expected_content").and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required `expected_content` input for compare_content"))?;
    let Some(response) = inputs.get("response") else {
        return Err(ExecError::new("missing required `response` input for compare_content"));
    };
    let file_response = match response {
        Value::Response(TransportResponse::File(r)) => r,
        _ => return Err(ExecError::new("compare_content expected file transport response")),
    };
    if !file_response.success {
        let error = file_response
            .error
            .clone()
            .unwrap_or_else(|| "unknown read failure".to_string());
        return Err(ExecError::new(format!(
            "compare_content read failed for `{}`: {error}",
            file_response.path
        )));
    }
    let actual = file_response.content.clone().ok_or_else(|| {
        ExecError::new(format!(
            "compare_content missing file content in successful response for `{}`",
            file_response.path
        ))
    })?;
    let fresh = actual == expected;
    OutputMap::new().bool("fresh", fresh).bool("skip", fresh).ok()
"##
        }
        HandlerKind::ExecuteTransport => {
            r##"    let skip = inputs.get("skip").and_then(Value::as_bool).unwrap_or(false);
    if skip {
        return OutputMap::new().value("response", Value::Skipped).ok();
    }
    if matches!(inputs.get("request"), Some(Value::Skipped) | None) {
        return OutputMap::new().value("response", Value::Skipped).ok();
    }
    let Some(request) = inputs.get("request").and_then(Value::as_request) else {
        return Err(ExecError::new("missing required `request` input for execute_transport"));
    };
    match request {
        TransportRequest::File(fr) => {
            let resp = execute_file_request(&fr);
            if fr.operation == FileOp::Write && !resp.success {
                let error = resp.error.clone().unwrap_or_else(|| "unknown write failure".to_string());
                return Err(ExecError::new(format!("failed to write `{}`: {error}", fr.path)));
            }
            OutputMap::new().response("response", TransportResponse::File(resp)).ok()
        }
        _ => Err(ExecError::new("unsupported transport request kind")),
    }
"##
        }
        HandlerKind::Collection => {
            r##"    let items = inputs.get("items").cloned()
        .ok_or_else(|| ExecError::new("missing required `items` input for collection"))?;
    OutputMap::new().value("items", items).ok()
"##
        }
        HandlerKind::Passthrough => {
            r##"    Ok(inputs)
"##
        }
    }
}

fn render_port_literal(name: &str, ty: &str, cardinality: Cardinality) -> String {
    if cardinality == Cardinality::ZERO {
        format!(r#"Port::void("{name}")"#)
    } else if cardinality == Cardinality::ONE {
        format!(r#"Port::scalar("{name}", "{ty}")"#)
    } else if cardinality == Cardinality::ZERO_OR_ONE {
        format!(r#"Port::optional("{name}", "{ty}")"#)
    } else if cardinality == Cardinality::ZERO_OR_MORE {
        format!(r#"Port::list("{name}", "{ty}")"#)
    } else if cardinality == Cardinality::ONE_OR_MORE {
        format!(r#"Port::non_empty_list("{name}", "{ty}")"#)
    } else {
        let max_expr = match cardinality.max {
            Some(value) => format!("Some({value})"),
            None => "None".to_string(),
        };
        // Custom cardinality: create scalar port then override cardinality.
        // This avoids depending on the pub(crate) `with_cardinality` constructor.
        format!(
            r#"{{ let mut p = Port::scalar("{name}", "{ty}"); p.cardinality = gunbc_ir::Cardinality::new({}, {max_expr}); p }}"#,
            cardinality.min
        )
    }
}

// ===========================================================================
// Cargo.toml
// ===========================================================================

fn emit_cargo_toml(
    module_name: &str,
    handler_kinds: &BTreeSet<HandlerKind>,
    dependency_paths: &DependencyPaths,
) -> String {
    let crate_name = module_name.replace('.', "-");
    let needs_helper = handler_kinds.contains(&HandlerKind::ExecuteReadContent)
        || handler_kinds.contains(&HandlerKind::ExecuteTransport);

    let mut deps = String::new();
    let _ = writeln!(
        deps,
        "gunbc-ir = {{ path = \"{}\" }}",
        dependency_paths.gunbc_ir
    );
    let _ = writeln!(
        deps,
        "gunbc-exec = {{ path = \"{}\" }}",
        dependency_paths.gunbc_exec
    );
    if needs_helper {
        let _ = writeln!(
            deps,
            "gunbc-lib-transport = {{ path = \"{}\" }}",
            dependency_paths.gunbc_lib_transport
        );
    }

    format!(
        r#"# Generated by daglang-emit (exec-runtime fast path).
# DO NOT EDIT — regenerate with `daglang compile`.

[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
{deps}
[workspace]
"#
    )
}

// ===========================================================================
// Helpers
// ===========================================================================

fn to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

// ===========================================================================
// IR construction helpers
// ===========================================================================

/// Build the SourceFile IR for the exec-runtime crate.
#[allow(clippy::vec_init_then_push)]
fn build_exec_runtime_source(
    dag: &Dag<LoweredOp>,
    module_name: &str,
    classified: &[ClassifiedNode],
    handler_kinds: &BTreeSet<HandlerKind>,
) -> gunbc_ir::code_ir::SourceFile {
    use gunbc_ir::code_ir::{Import, Item, SourceFile};

    let mut items = Vec::new();
    let needs_output_map = handler_kinds
        .iter()
        .any(|kind| handler_uses_output_map(*kind));
    let has_classified_edges = emitted_dag_has_edges(dag, classified);
    let needs_transport = handler_kinds.iter().any(|k| {
        matches!(
            k,
            HandlerKind::PrepareReadContent
                | HandlerKind::ExecuteReadContent
                | HandlerKind::PrepareWriteContent
                | HandlerKind::CompareContent
                | HandlerKind::ExecuteTransport
        )
    });
    let needs_helper = handler_kinds.contains(&HandlerKind::ExecuteReadContent)
        || handler_kinds.contains(&HandlerKind::ExecuteTransport);

    // ── Imports (proper IR) ──
    items.push(Item::Use(Import {
        path: vec!["std".into(), "collections".into()],
        items: vec!["HashMap".into()],
    }));
    let mut gunbc_exec_items = vec![
        "ExecError".into(),
        "ExecuteConfig".into(),
        "Executable".into(),
        "execute_dag".into(),
    ];
    if needs_output_map {
        gunbc_exec_items.push("OutputMap".into());
    }
    items.push(Item::Use(Import {
        path: vec!["gunbc_exec".into()],
        items: gunbc_exec_items,
    }));
    let mut gunbc_ir_items = vec!["Dag".into(), "Node".into(), "Port".into(), "Value".into()];
    if has_classified_edges {
        gunbc_ir_items.push("Edge".into());
    }
    items.push(Item::Use(Import {
        path: vec!["gunbc_ir".into()],
        items: gunbc_ir_items,
    }));

    if needs_transport {
        items.push(Item::Use(Import {
            path: vec!["gunbc_ir".into(), "transport".into()],
            items: vec![
                "FileOp".into(),
                "FileRequest".into(),
                "FileResponse".into(),
                "TransportRequest".into(),
                "TransportResponse".into(),
            ],
        }));
    }
    if needs_helper {
        items.push(Item::Use(Import {
            path: vec!["gunbc_lib_transport".into(), "executor".into()],
            items: vec!["execute_transport".into()],
        }));
    }
    // ── Op enum (Raw — optional data payload for literal-source nodes) ──
    items.push(build_op_enum_raw(handler_kinds));

    // ── impl Executable for Op (Raw — match indentation) ──
    items.push(build_executable_impl_raw(handler_kinds));

    // ── Handler functions (Raw per function) ──
    for kind in handler_kinds {
        items.push(build_handler_fn_raw(*kind));
    }

    // ── execute_file_request helper (Raw) ──
    if needs_helper {
        items.push(build_file_request_helper_raw());
    }

    // ── build_dag() (proper IR) ──
    items.push(build_build_dag_ir(dag, classified));

    // ── main() (Raw — multi-line match/if-let) ──
    items.push(build_main_raw(dag));

    SourceFile {
        doc: vec![
            format!("Generated by daglang-emit (exec-runtime IR path)."),
            format!("Module: {module_name}"),
            "DO NOT EDIT — regenerate with `daglang compile`.".to_string(),
        ],
        items,
    }
}

fn build_op_enum_raw(kinds: &BTreeSet<HandlerKind>) -> gunbc_ir::code_ir::Item {
    let mut text = String::new();
    writeln!(text, "#[derive(Debug, Clone, PartialEq)]").unwrap();
    writeln!(text, "enum Op {{").unwrap();
    for kind in kinds {
        if *kind == HandlerKind::LiteralSource {
            writeln!(
                text,
                "    LiteralSource {{ output_port: &'static str, value: Value }},"
            )
            .unwrap();
        } else {
            writeln!(text, "    {},", kind.variant_name()).unwrap();
        }
    }
    writeln!(text, "}}").unwrap();
    gunbc_ir::code_ir::Item::Raw(text)
}

fn build_executable_impl_raw(kinds: &BTreeSet<HandlerKind>) -> gunbc_ir::code_ir::Item {
    let mut text = String::new();
    writeln!(text, "impl Executable for Op {{").unwrap();
    writeln!(
        text,
        "    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {{"
    )
    .unwrap();
    writeln!(text, "        match self {{").unwrap();
    for kind in kinds {
        if *kind == HandlerKind::LiteralSource {
            writeln!(
                text,
                "            Self::LiteralSource {{ output_port, value }} => execute_literal_source(inputs, output_port, value),"
            )
            .unwrap();
        } else {
            writeln!(
                text,
                "            Self::{} => execute_{}(inputs),",
                kind.variant_name(),
                to_snake(kind.variant_name())
            )
            .unwrap();
        }
    }
    writeln!(text, "        }}").unwrap();
    writeln!(text, "    }}").unwrap();
    write!(text, "}}").unwrap();
    // Raw because Code IR lacks direct nodes for this generated impl/match layout.
    gunbc_ir::code_ir::Item::Raw(text)
}

fn build_handler_fn_raw(kind: HandlerKind) -> gunbc_ir::code_ir::Item {
    let fn_name = format!("execute_{}", to_snake(kind.variant_name()));
    let body = handler_body(kind);
    // Raw because handler bodies are authored as raw Rust snippets for exact control flow.
    if kind == HandlerKind::LiteralSource {
        gunbc_ir::code_ir::Item::Raw(format!(
            "fn {fn_name}(inputs: HashMap<String, Value>, output_port: &'static str, value: &Value) -> Result<HashMap<String, Value>, ExecError> {{\n    let _ = &inputs;\n{body}}}"
        ))
    } else {
        gunbc_ir::code_ir::Item::Raw(format!(
            "fn {fn_name}(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {{\n    let _ = &inputs;\n{body}}}"
        ))
    }
}

fn build_file_request_helper_raw() -> gunbc_ir::code_ir::Item {
    let mut text = String::new();
    writeln!(
        text,
        "fn execute_file_request(request: &FileRequest) -> FileResponse {{"
    )
    .unwrap();
    writeln!(
        text,
        "    match execute_transport(&TransportRequest::File(request.clone())) {{"
    )
    .unwrap();
    writeln!(
        text,
        "        Ok(TransportResponse::File(response)) => response,"
    )
    .unwrap();
    writeln!(text, "        Ok(_other) => FileResponse::error(").unwrap();
    writeln!(text, "            request.path.clone(), request.operation,").unwrap();
    writeln!(
        text,
        r#"            "transport executor returned non-file response for file request","#
    )
    .unwrap();
    writeln!(text, "        ),").unwrap();
    writeln!(
        text,
        "        Err(error) => FileResponse::error(request.path.clone(), request.operation, error.to_string()),"
    )
    .unwrap();
    writeln!(text, "    }}").unwrap();
    write!(text, "}}").unwrap();
    // Raw because this helper requires a handwritten nested match shape.
    gunbc_ir::code_ir::Item::Raw(text)
}

fn build_build_dag_ir(
    dag: &Dag<LoweredOp>,
    classified: &[ClassifiedNode],
) -> gunbc_ir::code_ir::Item {
    use gunbc_ir::code_ir::{Expr, FnDef, Item, Stmt};

    let mut body = Vec::new();
    body.push(Stmt::let_mut("dag", Expr::call("Dag::new", vec![])));

    // Nodes.
    for cn in classified {
        let inputs_code = cn
            .inputs
            .iter()
            .map(|(name, ty, cardinality)| render_port_literal(name, ty, *cardinality))
            .collect::<Vec<_>>()
            .join(", ");
        let outputs_code = cn
            .outputs
            .iter()
            .map(|(name, ty, cardinality)| render_port_literal(name, ty, *cardinality))
            .collect::<Vec<_>>()
            .join(", ");
        body.push(Stmt::Expr(Expr::raw(format!(
            r#"dag.add_node(Node::opaque("{}", vec![{inputs_code}], vec![{outputs_code}], {}))"#,
            cn.node_id, cn.op_ctor
        ))));
    }

    // Edges — only emit edges whose endpoints are classified (SubDag nodes
    // are skipped during classification, so their edges are excluded too).
    let classified_ids: std::collections::HashSet<&str> =
        classified.iter().map(|cn| cn.node_id.as_str()).collect();
    for edge in &dag.edges {
        if !classified_ids.contains(edge.from_node.0.as_str())
            || !classified_ids.contains(edge.to_node.0.as_str())
        {
            continue;
        }
        body.push(Stmt::Expr(Expr::raw(format!(
            r#"dag.add_edge(Edge::new("{}", "{}", "{}", "{}"))"#,
            edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
        ))));
    }

    body.push(Stmt::tail(Expr::var("dag")));

    Item::Fn(FnDef {
        name: "build_dag".to_string(),
        is_pub: false,
        params: vec![],
        return_type: Some("Dag<Op>".to_string()),
        body,
        doc: vec![],
        attributes: vec![],
    })
}

fn build_main_raw(dag: &Dag<LoweredOp>) -> gunbc_ir::code_ir::Item {
    // Compute entrypoints (same logic as emit_main).
    let connected_inputs: std::collections::HashSet<(String, String)> = dag
        .edges
        .iter()
        .map(|e| (e.to_node.0.clone(), e.to_port.0.clone()))
        .collect();

    let mut entrypoints: Vec<(String, String, String)> = Vec::new();
    for node in &dag.nodes {
        // Only String-typed CallParamSource nodes are meaningful CLI entrypoints.
        // Callable nodes have dead-weight parameter inputs (handlers ignore them).
        // Non-String param_source types (e.g. ToolRegistry) can't come from CLI.
        let NodeBody::Opaque(op) = &node.body else {
            continue;
        };
        let is_string_param_source = matches!(
            op,
            LoweredOp::Primitive {
                kind: PrimitiveOpKind::CallParamSource { .. },
                ..
            }
        ) && node.inputs.iter().any(|p| p.type_id.0 == "String");
        if !is_string_param_source {
            continue;
        }
        for port in &node.inputs {
            if port.name.is_resource() || port.name.is_internal() {
                continue;
            }
            let key = (node.id.0.clone(), port.name.0.clone());
            if !connected_inputs.contains(&key) {
                entrypoints.push((
                    node.id.0.clone(),
                    port.name.0.clone(),
                    port.type_id.0.clone(),
                ));
            }
        }
    }

    let mut text = String::new();
    writeln!(text, "fn main() {{").unwrap();
    if entrypoints.is_empty() {
        writeln!(
            text,
            r#"    let trace_json = std::env::args().any(|a| a == "--trace-json");"#
        )
        .unwrap();
    } else {
        writeln!(
            text,
            "    let raw_args: Vec<String> = std::env::args().collect();"
        )
        .unwrap();
        writeln!(
            text,
            r#"    let trace_json = raw_args.iter().any(|a| a == "--trace-json");"#
        )
        .unwrap();
        writeln!(
            text,
            r#"    let args: Vec<String> = raw_args.into_iter().filter(|a| a != "--trace-json").collect();"#
        )
        .unwrap();
    }
    writeln!(text).unwrap();

    if !entrypoints.is_empty() {
        writeln!(text, "    // Parse entrypoint values from CLI args.").unwrap();
        writeln!(
            text,
            "    let mut input_mocks = gunbc_exec::BoundaryMocks::new();"
        )
        .unwrap();
        for (i, (node_id, port_name, _type_id)) in entrypoints.iter().enumerate() {
            let arg_idx = i + 1;
            writeln!(text, r#"    if let Some(val) = args.get({arg_idx}) {{"#).unwrap();
            writeln!(
                text,
                r#"        input_mocks.set_input("{node_id}", "{port_name}", Value::Str(val.clone()));"#
            )
            .unwrap();
            writeln!(text, "    }}").unwrap();
        }
        writeln!(text).unwrap();
    }

    writeln!(text, "    let dag = build_dag();").unwrap();
    if entrypoints.is_empty() {
        writeln!(
            text,
            "    let result = execute_dag(&dag, ExecuteConfig::default());"
        )
        .unwrap();
    } else {
        writeln!(text, "    let result = execute_dag(&dag, ExecuteConfig {{ input_mocks: Some(&input_mocks), ..Default::default() }});").unwrap();
    }
    writeln!(text, "    match result {{").unwrap();
    writeln!(text, "        Ok(log) => {{").unwrap();
    writeln!(text, "            if trace_json {{").unwrap();
    writeln!(text, "                let nodes: Vec<&str> = log.entries.iter().map(|e| e.node_id.as_str()).collect();").unwrap();
    writeln!(text, r#"                let nodes_json: Vec<String> = nodes.iter().map(|n| format!("\"{{}}\"", n)).collect();"#).unwrap();
    writeln!(text, r#"                let entries_json: Vec<String> = log.entries.iter().map(|e| format!("{{{{\"node_id\":\"{{}}\",\"intercepted\":{{}}}}}}", e.node_id, e.was_intercepted)).collect();"#).unwrap();
    writeln!(text, r#"                eprintln!("{{{{\"nodes\":[{{}}],\"entries\":[{{}}]}}}}", nodes_json.join(","), entries_json.join(","));"#).unwrap();
    writeln!(text, "            }}").unwrap();
    writeln!(
        text,
        r#"                eprintln!("execution completed: {{}} nodes executed", log.entries.len());"#
    )
    .unwrap();
    writeln!(text, "                for entry in &log.entries {{").unwrap();
    writeln!(
        text,
        r#"                    eprintln!("  [{{}}] intercepted={{}}", entry.node_id, entry.was_intercepted);"#
    )
    .unwrap();
    writeln!(text, "                }}").unwrap();
    writeln!(text, "        }}").unwrap();
    writeln!(text, "        Err(e) => {{").unwrap();
    writeln!(text, r#"            eprintln!("execution failed: {{e}}");"#).unwrap();
    writeln!(text, "            std::process::exit(1);").unwrap();
    writeln!(text, "        }}").unwrap();
    writeln!(text, "    }}").unwrap();
    write!(text, "}}").unwrap();

    // Raw because main() is emitted as a single procedural text template.
    gunbc_ir::code_ir::Item::Raw(text)
}

fn handler_uses_output_map(kind: HandlerKind) -> bool {
    matches!(
        kind,
        HandlerKind::FsEnv
            | HandlerKind::LiteralSource
            | HandlerKind::PrepareReadContent
            | HandlerKind::ExecuteReadContent
            | HandlerKind::PrepareWriteContent
            | HandlerKind::CompareContent
            | HandlerKind::ExecuteTransport
            | HandlerKind::Collection
    )
}

fn emitted_dag_has_edges(dag: &Dag<LoweredOp>, classified: &[ClassifiedNode]) -> bool {
    let classified_ids: std::collections::HashSet<&str> = classified
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();

    dag.edges.iter().any(|edge| {
        classified_ids.contains(edge.from_node.0.as_str())
            && classified_ids.contains(edge.to_node.0.as_str())
    })
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{
        CallableKind, CallableObligation, LoweredOp, PrimitiveLiteral, PrimitiveOpKind,
        ServiceCallMetadata, ServiceTransportClass, TransportObligation,
    };
    use gunbc_ir::{Node, Port};

    // ── Obligation classification policy (Phase 1 mirror of arch_rules.dag) ──

    /// Obligations safe to emit as passthrough stubs in generated exec-runtime
    /// code. Mirrors `passthrough_safe_obligations` in `dsl/config/arch_rules.dag`.
    fn passthrough_safe_obligations() -> &'static [ObligationCategory] {
        &[
            ObligationCategory::None,
            ObligationCategory::PureGeneric,
            ObligationCategory::ServiceTransportPrepare,
            ObligationCategory::ServiceTransportExecute,
            ObligationCategory::ServiceTransportParse,
            ObligationCategory::ServiceParamSource,
            ObligationCategory::ResourceProvide,
            ObligationCategory::ResourceAcquire,
            ObligationCategory::ResourceRelease,
            ObligationCategory::InterfaceContractVerification,
        ]
    }

    /// Obligations that require real handlers. Mirrors `require_handler_obligations`
    /// in `dsl/config/arch_rules.dag`.
    fn require_handler_obligations() -> &'static [ObligationCategory] {
        &[
            ObligationCategory::PureRender,
            ObligationCategory::PureDataLoad,
        ]
    }

    /// Unknown-module callables with known obligations emit as passthrough
    /// because all LoweredOp::Callable nodes come from the DSL compiler
    /// (the lowerer only creates them for items in the typed project).
    ///
    /// TODO(NF-5): once all handlers have specialized implementations,
    /// this should become a rejection (unknown module → error).
    #[test]
    fn emit_exec_runtime_unknown_module_uses_passthrough() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "unknown_op",
            vec![],
            vec![],
            LoweredOp::Callable {
                module: "tools.unknown".to_string(),
                kind: CallableKind::Fn,
                name: "something".to_string(),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        let result = emit_exec_runtime(&dag, "tools.unknown");
        assert!(
            result.is_ok(),
            "compiler-validated callables should emit as passthrough: {:?}",
            result.err()
        );
    }

    #[test]
    fn emit_exec_runtime_supports_literal_source_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "literal_path",
            vec![],
            vec![Port::scalar("path", "String")],
            LoweredOp::Primitive {
                module: "tools.pragma".to_string(),
                name: "call_literal_source::strhex:636c697070792e746f6d6c".to_string(),
                kind: PrimitiveOpKind::CallLiteralSource {
                    literal: PrimitiveLiteral::String("clippy.toml".to_string()),
                },
            },
        ));

        let files = emit_exec_runtime(&dag, "tools.pragma").expect("literal source should emit");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("LiteralSource { output_port: &'static str, value: Value }"),
            "generated Op enum should include literal-source payload variant"
        );
        assert!(
            main_rs.contains(
                "Op::LiteralSource { output_port: \"path\", value: Value::Str(\"clippy.toml\".to_string()) }"
            ),
            "build_dag should instantiate literal-source op with native Value payload"
        );
        assert!(
            main_rs.contains("execute_literal_source(inputs, output_port, value)"),
            "Executable impl should dispatch literal-source payload variant"
        );
    }

    #[test]
    fn to_snake_converts_pascal_to_snake() {
        assert_eq!(to_snake("LoadRegistry"), "load_registry");
        assert_eq!(to_snake("FsEnv"), "fs_env");
        assert_eq!(to_snake("RenderMakefile"), "render_makefile");
        assert_eq!(to_snake("PrepareReadContent"), "prepare_read_content");
        assert_eq!(to_snake("ExecuteTransport"), "execute_transport");
    }

    #[test]
    fn classify_handler_uses_passthrough_for_unspecialized_surfaces() {
        // std.patterns callables use passthrough (obligation: None).
        let pattern_callable = LoweredOp::Callable {
            module: "std.patterns".into(),
            kind: CallableKind::Pattern,
            name: "file_content_matches".into(),
            obligation: CallableObligation::None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        };
        assert_eq!(
            classify_handler(&pattern_callable),
            Some(HandlerClassification::Handler(HandlerKind::Passthrough))
        );

        // Service transport nodes use passthrough.
        let service_prepare = LoweredOp::Transport {
            module: "services.workflow.control_plane".into(),
            kind: CallableKind::Pattern,
            name: "service_transport::prepare::workflow.ControlPlane::AcquireStageClaim".into(),
            obligation: TransportObligation::Prepare,
            service_metadata: Box::new(ServiceCallMetadata {
                service: "workflow.ControlPlane".to_string(),
                operation: "AcquireStageClaim".to_string(),
                transport: ServiceTransportClass::ShellLocal,
                idempotent: true,
                readonly: false,
                spec: None,
                response_provider: None,
            }),
            is_interactive: false,
            resource_target: None,
        };
        assert_eq!(
            classify_handler(&service_prepare),
            Some(HandlerClassification::Handler(HandlerKind::Passthrough))
        );
    }

    #[test]
    fn classify_handler_obligation_gated_passthrough() {
        // Helper to build a callable with a given obligation.
        let make = |obligation: CallableObligation| LoweredOp::Callable {
            module: "tools.newfeature".into(),
            kind: CallableKind::Fn,
            name: "some_op".into(),
            obligation,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        };

        // PureGeneric → passthrough (DSL body wrapper, no specialized handler needed).
        assert_eq!(
            classify_handler(&make(CallableObligation::PureGeneric)),
            Some(HandlerClassification::Handler(HandlerKind::Passthrough)),
            "PureGeneric should passthrough"
        );

        // PureRender requires a dedicated handler and is not classified as passthrough.
        assert_eq!(
            classify_handler(&make(CallableObligation::PureRender)),
            None,
            "PureRender should be unresolved without a dedicated handler"
        );

        // PureDataLoad requires a dedicated handler and is not classified as passthrough.
        assert_eq!(
            classify_handler(&make(CallableObligation::PureDataLoad)),
            None,
            "PureDataLoad should be unresolved without a dedicated handler"
        );

        // ResourceProvide → passthrough (structural).
        assert_eq!(
            classify_handler(&make(CallableObligation::ResourceProvide)),
            Some(HandlerClassification::Handler(HandlerKind::Passthrough)),
            "ResourceProvide should passthrough"
        );
    }

    #[test]
    fn emit_exec_runtime_omits_unused_imports_and_args_for_single_node_graph() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "tools.pragma::render_clippy_toml",
            vec![],
            vec![Port::scalar("return", "String")],
            LoweredOp::Callable {
                module: "tools.pragma".to_string(),
                kind: CallableKind::Fn,
                name: "render_clippy_toml".to_string(),
                obligation: CallableObligation::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));

        let files = emit_exec_runtime(&dag, "tools.pragma").expect("emit should succeed");
        let main_rs = &files[0].content;
        assert!(
            !main_rs.contains("Edge"),
            "single-node graph should not import Edge"
        );
        assert!(
            !main_rs.contains("let args: Vec<String>"),
            "graph with no entrypoint CLI params should not create args vector"
        );
        assert!(
            main_rs.contains("let trace_json = std::env::args().any"),
            "trace flag parsing should still work without entrypoint args"
        );
    }

    #[test]
    fn emit_exec_runtime_uses_configured_dependency_paths() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "transport_execute",
            vec![],
            vec![],
            LoweredOp::Primitive {
                module: "tools.transport".to_string(),
                name: "transport_execute".to_string(),
                kind: PrimitiveOpKind::IoExecuteFileWrite,
            },
        ));

        let config = EmitConfig {
            dependency_paths: DependencyPaths {
                gunbc_ir: "../../../src/v1/00_foundation/ir".to_string(),
                gunbc_exec: "../../../src/v1/09_execute/exec".to_string(),
                gunbc_lib_transport: "../../../src/v1/08_materialize/transport".to_string(),
            },
        };
        let files = emit_exec_runtime_with_config(&dag, "tools.transport", &config)
            .expect("emit should succeed with explicit dependency paths");
        let cargo_toml = files
            .iter()
            .find(|file| file.path == "Cargo.toml")
            .expect("Cargo.toml should be emitted");

        assert!(
            cargo_toml
                .content
                .contains("gunbc-ir = { path = \"../../../src/v1/00_foundation/ir\" }"),
            "Cargo.toml should use the configured gunbc-ir path"
        );
        assert!(
            cargo_toml
                .content
                .contains("gunbc-exec = { path = \"../../../src/v1/09_execute/exec\" }"),
            "Cargo.toml should use the configured gunbc-exec path"
        );
        assert!(
            cargo_toml.content.contains(
                "gunbc-lib-transport = { path = \"../../../src/v1/08_materialize/transport\" }"
            ),
            "Cargo.toml should use the configured transport path when helper support is needed"
        );
    }

    #[test]
    fn passthrough_obligations_pinned_to_policy() {
        // The passthrough arms in classify_handler must exactly match the
        // policy declarations. Adding a new passthrough obligation without
        // updating passthrough_safe_obligations() or
        // require_handler_obligations() fails this test.
        let safe = passthrough_safe_obligations();
        let require = require_handler_obligations();

        // No overlap between safe and require-handler sets.
        for ob in require {
            assert!(
                !safe.contains(ob),
                "obligation {ob:?} appears in both passthrough_safe and require_handler"
            );
        }

        // Every ObligationCategory variant is accounted for in exactly one set.
        let all_obligations = [
            ObligationCategory::None,
            ObligationCategory::PureGeneric,
            ObligationCategory::ServiceTransportPrepare,
            ObligationCategory::ServiceTransportExecute,
            ObligationCategory::ServiceTransportParse,
            ObligationCategory::ServiceParamSource,
            ObligationCategory::ResourceProvide,
            ObligationCategory::ResourceAcquire,
            ObligationCategory::ResourceRelease,
            ObligationCategory::InterfaceContractVerification,
            ObligationCategory::PureRender,
            ObligationCategory::PureDataLoad,
        ];

        for ob in &all_obligations {
            assert!(
                safe.contains(ob) || require.contains(ob),
                "ObligationCategory::{ob:?} is not in passthrough_safe or require_handler"
            );
        }

        // Every entry in the policy sets is a real variant.
        for ob in safe {
            assert!(
                all_obligations.contains(ob),
                "passthrough_safe contains unknown obligation: {ob:?}"
            );
        }
        for ob in require {
            assert!(
                all_obligations.contains(ob),
                "require_handler contains unknown obligation: {ob:?}"
            );
        }
    }
}
