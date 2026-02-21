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
//! - `fn main()` with CLI arg parsing + `execute_with_mode_and_inputs`
//! - `Cargo.toml` with `gunbc-ir`/`gunbc-exec`/`gunbc-lib-transport` deps

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use daglang_lower::{LoweredOp, PrimitiveLiteral, PrimitiveOpKind};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Dag;
use gunbc_ir::{Cardinality, WorkspaceLayout};

use crate::EmittedFile;

// ===========================================================================
// Public API
// ===========================================================================

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
    emit_exec_runtime_with_output_dir(dag, module_name, None)
}

/// Emit a standalone Rust crate from a lowered DAG with an optional output directory.
///
/// When `output_dir` is provided, Cargo path dependencies are rendered relative
/// to that directory using workspace layout discovery.
pub fn emit_exec_runtime_with_output_dir(
    dag: &Dag<LoweredOp>,
    module_name: &str,
    output_dir: Option<&Path>,
) -> Result<Vec<EmittedFile>, ExecRuntimeError> {
    let classified = classify_nodes(dag)?;
    let handler_kinds = collect_handler_kinds(&classified);
    let source = build_exec_runtime_source(dag, module_name, &classified, &handler_kinds);
    let main_rs = crate::render_rust::render_rust_source(&source);
    let cargo_toml = emit_cargo_toml(module_name, &handler_kinds, output_dir);

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
    LoadRegistry,
    FsEnv,
    RenderMakefile,
    Entrypoint,
    InfraEntrypoint,
    ParamSource,
    LiteralSource,
    RenderPragmaClippyToml,
    RenderPragmaAllowlist,
    RenderPragmaLintPolicy,
    PragmaEntrypoint,
    PrepareReadContent,
    ExecuteReadContent,
    PrepareWriteContent,
    CompareContent,
    ExecuteTransport,
    Collection,
}

impl HandlerKind {
    fn variant_name(self) -> &'static str {
        match self {
            Self::LoadRegistry => "LoadRegistry",
            Self::FsEnv => "FsEnv",
            Self::RenderMakefile => "RenderMakefile",
            Self::Entrypoint => "Entrypoint",
            Self::InfraEntrypoint => "InfraEntrypoint",
            Self::ParamSource => "ParamSource",
            Self::LiteralSource => "LiteralSource",
            Self::RenderPragmaClippyToml => "RenderPragmaClippyToml",
            Self::RenderPragmaAllowlist => "RenderPragmaAllowlist",
            Self::RenderPragmaLintPolicy => "RenderPragmaLintPolicy",
            Self::PragmaEntrypoint => "PragmaEntrypoint",
            Self::PrepareReadContent => "PrepareReadContent",
            Self::ExecuteReadContent => "ExecuteReadContent",
            Self::PrepareWriteContent => "PrepareWriteContent",
            Self::CompareContent => "CompareContent",
            Self::ExecuteTransport => "ExecuteTransport",
            Self::Collection => "Collection",
        }
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

fn classify_nodes(dag: &Dag<LoweredOp>) -> Result<Vec<ClassifiedNode>, ExecRuntimeError> {
    let mut result = Vec::with_capacity(dag.nodes.len());
    for node in &dag.nodes {
        let node_id = node.id.0.clone();

        let op = match &node.body {
            NodeBody::Opaque(op) => op,
            NodeBody::SubDag(_) => {
                return Err(ExecRuntimeError::SubDagNotSupported { node_id });
            }
        };

        let handler = classify_handler(op).ok_or_else(|| ExecRuntimeError::UnresolvableNode {
            node_id: node_id.clone(),
            detail: format!("no runtime op classification for {op:?}"),
        })?;
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

fn classify_handler(op: &LoweredOp) -> Option<HandlerKind> {
    match op {
        LoweredOp::Collection { .. } => return Some(HandlerKind::Collection),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CallParamSource { .. },
            ..
        } => return Some(HandlerKind::ParamSource),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CallLiteralSource { .. },
            ..
        } => return Some(HandlerKind::LiteralSource),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::FsEnv,
            ..
        } => return Some(HandlerKind::FsEnv),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoPrepareFileRead,
            ..
        } => return Some(HandlerKind::PrepareReadContent),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileRead,
            ..
        } => return Some(HandlerKind::ExecuteReadContent),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoPrepareFileWrite,
            ..
        } => return Some(HandlerKind::PrepareWriteContent),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::CompareEquality,
            ..
        } => return Some(HandlerKind::CompareContent),
        LoweredOp::Primitive {
            kind: PrimitiveOpKind::IoExecuteFileWrite,
            ..
        } => return Some(HandlerKind::ExecuteTransport),
        LoweredOp::Pipeline { .. } => {}
        LoweredOp::Callable { module, name, .. } if module == "tools.makegen" => {
            return match name.as_str() {
                "load_registry" => Some(HandlerKind::LoadRegistry),
                "render_makefile" => Some(HandlerKind::RenderMakefile),
                "makegen" => Some(HandlerKind::Entrypoint),
                _ => None,
            };
        }
        LoweredOp::Callable { name, .. }
            if name.starts_with("service_transport::prepare::")
                || name.starts_with("service_transport::parse::")
                || name.starts_with("service_transport::execute::") =>
        {
            return Some(HandlerKind::ParamSource);
        }
        LoweredOp::Callable { .. } => {}
    }

    let (module, name) = match op {
        LoweredOp::Callable { module, name, .. } => (module.as_str(), name.as_str()),
        LoweredOp::Pipeline { module, name, .. } => (module.as_str(), name.as_str()),
        _ => return None,
    };

    match (module, name) {
        ("tools.infra", "infra") => Some(HandlerKind::InfraEntrypoint),
        ("pipelines.sdlc", _) => Some(HandlerKind::ParamSource),
        ("tools.design", _) => Some(HandlerKind::ParamSource),
        ("shared.dag_util", _) => Some(HandlerKind::ParamSource),
        ("std.patterns", _) => Some(HandlerKind::ParamSource),
        ("std.resources", _) => Some(HandlerKind::ParamSource),
        (module, _) if module.starts_with("services.") => Some(HandlerKind::ParamSource),
        ("tools.pragma", "render_clippy_toml") => Some(HandlerKind::RenderPragmaClippyToml),
        ("tools.pragma", "render_disallowed_methods_allowlist") => {
            Some(HandlerKind::RenderPragmaAllowlist)
        }
        ("tools.pragma", "render_pragma_lint_policy") => Some(HandlerKind::RenderPragmaLintPolicy),
        ("tools.pragma", "pragma") => Some(HandlerKind::PragmaEntrypoint),
        _ => None,
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
        PrimitiveLiteral::Unit => "Value::Unit".to_string(),
    }
}

// ===========================================================================
// Handler helpers (shared by IR path)
// ===========================================================================

fn requires_pragma_helpers(kinds: &BTreeSet<HandlerKind>) -> bool {
    kinds.contains(&HandlerKind::RenderPragmaClippyToml)
        || kinds.contains(&HandlerKind::RenderPragmaAllowlist)
        || kinds.contains(&HandlerKind::RenderPragmaLintPolicy)
}

fn emit_pragma_helpers(out: &mut String) {
    out.push_str(
        r#"
#[derive(Debug, Clone)]
struct PragmaDirectiveRuntime {
    key: String,
    value: String,
    scope: Option<String>,
}

fn parse_pragma_directives(value: &Value) -> Result<Vec<PragmaDirectiveRuntime>, ExecError> {
    match value {
        Value::Str(raw) => {
            let parsed: serde_json::Value = serde_json::from_str(raw)
                .map_err(|error| ExecError::new(format!("invalid directives JSON: {error}")))?;
            parse_pragma_directives_json(&parsed)
        }
        Value::Json(json) => parse_pragma_directives_json(json),
        Value::List(items) => parse_pragma_directives_list(items),
        _ => Err(ExecError::new(
            "missing or invalid `directives` input; expected JSON array/list",
        )),
    }
}

fn parse_pragma_directives_json(
    json: &serde_json::Value,
) -> Result<Vec<PragmaDirectiveRuntime>, ExecError> {
    let entries = json
        .as_array()
        .ok_or_else(|| ExecError::new("`directives` JSON must be an array"))?;
    let mut directives = Vec::with_capacity(entries.len());
    for entry in entries {
        let obj = entry
            .as_object()
            .ok_or_else(|| ExecError::new("each directive JSON entry must be an object"))?;
        let key = obj
            .get("key")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ExecError::new("directive JSON entry missing string `key`"))?
            .to_string();
        let value = obj
            .get("value")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ExecError::new("directive JSON entry missing string `value`"))?
            .to_string();
        let scope = match obj.get("scope") {
            Some(v) if v.is_null() => None,
            Some(v) => Some(
                v.as_str()
                    .ok_or_else(|| {
                        ExecError::new("directive JSON entry `scope` must be string or null")
                    })?
                    .to_string(),
            ),
            None => None,
        };
        directives.push(PragmaDirectiveRuntime { key, value, scope });
    }
    Ok(directives)
}

fn parse_pragma_directives_list(
    items: &[Value],
) -> Result<Vec<PragmaDirectiveRuntime>, ExecError> {
    let mut directives = Vec::with_capacity(items.len());
    for item in items {
        let map = item
            .as_map()
            .ok_or_else(|| ExecError::new("each `directives` list entry must be a map"))?;
        let key = map
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| ExecError::new("directive list entry missing string `key`"))?
            .to_string();
        let value = map
            .get("value")
            .and_then(Value::as_str)
            .ok_or_else(|| ExecError::new("directive list entry missing string `value`"))?
            .to_string();
        let scope = match map.get("scope") {
            Some(Value::Str(scope)) => Some(scope.clone()),
            Some(Value::Unit) | None => None,
            Some(_) => {
                return Err(ExecError::new(
                    "directive list entry `scope` must be string or unit",
                ));
            }
        };
        directives.push(PragmaDirectiveRuntime { key, value, scope });
    }
    Ok(directives)
}

"#,
    );
}

/// Return the body (inside braces) for each handler kind.
///
/// Uses `r##"..."##` to avoid issues with inner `"#` sequences.
fn handler_body(kind: HandlerKind) -> &'static str {
    match kind {
        HandlerKind::LoadRegistry => {
            r##"    OutputMap::new().str("registry", "{}").ok()
"##
        }
        HandlerKind::FsEnv => {
            r##"    OutputMap::new().str("FilesystemHandle", "filesystem://workspace").ok()
"##
        }
        HandlerKind::RenderMakefile => {
            r##"    let content = include_str!("embedded_makefile.txt");
    OutputMap::new().str("return", content.to_string()).ok()
"##
        }
        HandlerKind::Entrypoint => {
            r##"    let written = inputs.get("__deps").and_then(Value::as_list).map(|deps| {
        deps.iter().any(|value| matches!(value,
            Value::Response(TransportResponse::File(r)) if r.operation == FileOp::Write && r.success))
    }).unwrap_or(false);
    OutputMap::new().bool("written", written).ok()
"##
        }
        HandlerKind::InfraEntrypoint => {
            r##"    let environment = inputs.get("environment")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required input `environment`"))?;
    let runtime = inputs.get("runtime")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required input `runtime`"))?;
    let parse_csv_list = |raw: &str| -> Vec<String> {
        raw.split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect()
    };
    let spec_targets = match inputs.get("spec_targets") {
        Some(value) => value
            .as_str_list()
            .or_else(|| value.as_str().map(parse_csv_list))
            .ok_or_else(|| ExecError::new("invalid input `spec_targets` (expected list or CSV string)"))?,
        None => return Err(ExecError::new("missing required input `spec_targets`")),
    };
    let target = inputs
        .get("target")
        .and_then(|value| value.as_str_list().or_else(|| value.as_str().map(parse_csv_list)))
        .unwrap_or_default();
    let skip = inputs
        .get("skip")
        .and_then(|value| value.as_str_list().or_else(|| value.as_str().map(parse_csv_list)))
        .unwrap_or_default();
    let execute = match inputs.get("execute") {
        Some(value) => value.as_bool().or_else(|| {
            value.as_str().and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
                "true" | "1" => Some(true),
                "false" | "0" => Some(false),
                _ => None,
            })
        })
        .ok_or_else(|| ExecError::new("invalid input `execute` (expected bool or true/false string)"))?,
        None => return Err(ExecError::new("missing required input `execute`")),
    };
    let mut planned_targets = if target.is_empty() {
        spec_targets.clone()
    } else {
        spec_targets
            .iter()
            .filter(|item| target.iter().any(|candidate| candidate == *item))
            .cloned()
            .collect::<Vec<_>>()
    };
    planned_targets.retain(|item| !skip.iter().any(|candidate| candidate == item));
    let target_count = planned_targets.len() as i64;
    let applied_count = if execute { target_count } else { 0 };
    let mode = if execute { "apply" } else { "plan" };
    let report = format!(
        "infra {mode} (env={environment}, runtime={runtime}): {target_count} target(s)"
    );
    OutputMap::new()
        .str("environment", environment)
        .str("runtime", runtime)
        .str("mode", mode)
        .str_list("planned_targets", planned_targets)
        .int("target_count", target_count)
        .int("applied_count", applied_count)
        .str("report", report)
        .ok()
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
        HandlerKind::RenderPragmaClippyToml => {
            r##"    let directives_value = inputs.get("directives")
        .ok_or_else(|| ExecError::new("missing required input `directives`"))?;
    let directives = parse_pragma_directives(directives_value)?;
    let header = "# Generated by gunbc pragma\n";
    let entries = directives
        .iter()
        .filter(|directive| {
            directive.scope.as_deref() == Some("clippy") || directive.scope.is_none()
        })
        .map(|directive| format!("{} = {}", directive.key, directive.value))
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!("{header}\n{entries}\n");
    OutputMap::new().str("return", content).ok()
"##
        }
        HandlerKind::RenderPragmaAllowlist => {
            r##"    let directives_value = inputs.get("directives")
        .ok_or_else(|| ExecError::new("missing required input `directives`"))?;
    let directives = parse_pragma_directives(directives_value)?;
    let header = "# Generated by gunbc pragma\n";
    let methods = directives
        .iter()
        .filter(|directive| directive.key == "disallowed_method")
        .map(|directive| directive.value.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!("{header}{methods}\n");
    OutputMap::new().str("return", content).ok()
"##
        }
        HandlerKind::RenderPragmaLintPolicy => {
            r##"    let directives_value = inputs.get("directives")
        .ok_or_else(|| ExecError::new("missing required input `directives`"))?;
    let directives = parse_pragma_directives(directives_value)?;
    let header = "# Generated by gunbc pragma\n";
    let policies = directives
        .iter()
        .filter(|directive| directive.scope.as_deref() == Some("lint"))
        .map(|directive| format!("{}: {}", directive.key, directive.value))
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!("{header}{policies}\n");
    OutputMap::new().str("return", content).ok()
"##
        }
        HandlerKind::PragmaEntrypoint => {
            r##"    let deps = inputs.get("__deps").and_then(Value::as_list).unwrap_or(&[]);
    let mut clippy_written = false;
    let mut allowlist_written = false;
    let mut policy_written = false;
    for dep in deps {
        let Value::Response(TransportResponse::File(response)) = dep else {
            continue;
        };
        if response.operation != FileOp::Write || !response.success {
            continue;
        }
        if response.path.ends_with("clippy.toml") {
            clippy_written = true;
        } else if response.path.ends_with("tools/disallowed-methods-allowlist.txt") {
            allowlist_written = true;
        } else if response.path.ends_with("tools/pragma-lint-policy.txt") {
            policy_written = true;
        }
    }
    OutputMap::new()
        .bool("clippy_written", clippy_written)
        .bool("allowlist_written", allowlist_written)
        .bool("policy_written", policy_written)
        .ok()
"##
        }
        HandlerKind::PrepareReadContent => {
            r##"    let path = inputs.get("path").and_then(Value::as_str).unwrap_or("");
    if path.is_empty() {
        return OutputMap::new().value("request", Value::Skipped).ok();
    }
    OutputMap::new().request("request", TransportRequest::File(FileRequest::read(path))).ok()
"##
        }
        HandlerKind::ExecuteReadContent => {
            r##"    let Some(request) = inputs.get("request").and_then(Value::as_request) else {
        return OutputMap::new().value("response", Value::Skipped).ok();
    };
    match request {
        TransportRequest::File(fr) => {
            let resp = execute_file_request(&fr);
            OutputMap::new().response("response", TransportResponse::File(resp)).ok()
        }
        _ => OutputMap::new().value("response", Value::Skipped).ok(),
    }
"##
        }
        HandlerKind::PrepareWriteContent => {
            r##"    let path = inputs.get("path").and_then(Value::as_str).unwrap_or("");
    let content = inputs.get("content").and_then(Value::as_str).unwrap_or("");
    if path.is_empty() || content.is_empty() {
        return OutputMap::new().value("request", Value::Skipped).ok();
    }
    OutputMap::new().request("request", TransportRequest::File(FileRequest::write(path, content))).ok()
"##
        }
        HandlerKind::CompareContent => {
            r##"    let expected = inputs.get("expected_content").and_then(Value::as_str).unwrap_or("");
    let actual = match inputs.get("response") {
        Some(Value::Response(TransportResponse::File(r))) if r.success => {
            r.content.clone().unwrap_or_default()
        }
        _ => String::new(),
    };
    let fresh = actual == expected;
    OutputMap::new().bool("fresh", fresh).bool("skip", fresh).ok()
"##
        }
        HandlerKind::ExecuteTransport => {
            r##"    let skip = inputs.get("skip").and_then(Value::as_bool).unwrap_or(false);
    if skip {
        return OutputMap::new().value("response", Value::Skipped).ok();
    }
    let Some(request) = inputs.get("request").and_then(Value::as_request) else {
        return OutputMap::new().value("response", Value::Skipped).ok();
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
        _ => OutputMap::new().value("response", Value::Skipped).ok(),
    }
"##
        }
        HandlerKind::Collection => {
            r##"    let items = inputs.get("items").cloned().unwrap_or(Value::Skipped);
    OutputMap::new().value("items", items).ok()
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
        format!(
            r#"Port::with_cardinality("{name}", "{ty}", gunbc_ir::Cardinality::new({}, {max_expr}))"#,
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
    output_dir: Option<&Path>,
) -> String {
    let crate_name = module_name.replace('.', "-");
    let needs_helper = handler_kinds.contains(&HandlerKind::ExecuteReadContent)
        || handler_kinds.contains(&HandlerKind::ExecuteTransport);
    let needs_serde_json = requires_pragma_helpers(handler_kinds);
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .ok();

    let mut deps = String::new();
    let _ = writeln!(
        deps,
        "gunbc-ir = {{ path = \"{}\" }}",
        dependency_path(layout.as_ref(), output_dir, "gunbc-ir", "../../core/ir")
    );
    let _ = writeln!(
        deps,
        "gunbc-exec = {{ path = \"{}\" }}",
        dependency_path(layout.as_ref(), output_dir, "gunbc-exec", "../../core/exec")
    );
    if needs_helper {
        let _ = writeln!(
            deps,
            "gunbc-lib-transport = {{ path = \"{}\" }}",
            dependency_path(
                layout.as_ref(),
                output_dir,
                "gunbc-lib-transport",
                "../../lib/transport"
            )
        );
    }
    if needs_serde_json {
        deps.push_str("serde_json = \"1\"\n");
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

fn dependency_path(
    layout: Option<&WorkspaceLayout>,
    output_dir: Option<&Path>,
    crate_name: &str,
    fallback: &str,
) -> String {
    let Some(layout) = layout else {
        return fallback.to_string();
    };
    let Some(output_dir) = output_dir else {
        return fallback.to_string();
    };
    let Some(dep_dir) = layout.crate_dir(crate_name) else {
        return fallback.to_string();
    };
    normalize_dep_path(&layout.relative_path(output_dir, dep_dir))
}

fn normalize_dep_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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

    // ── Imports (proper IR) ──
    items.push(Item::Use(Import {
        path: vec!["std".into(), "collections".into()],
        items: vec!["HashMap".into()],
    }));
    items.push(Item::Use(Import {
        path: vec!["gunbc_exec".into()],
        items: vec![
            "ExecError".into(),
            "Executable".into(),
            "ExecutionMode".into(),
            "execute_with_mode_and_inputs".into(),
            "OutputMap".into(),
        ],
    }));
    items.push(Item::Use(Import {
        path: vec!["gunbc_ir".into()],
        items: vec![
            "Dag".into(),
            "Edge".into(),
            "Node".into(),
            "Port".into(),
            "Value".into(),
        ],
    }));
    let needs_transport = handler_kinds.iter().any(|k| {
        matches!(
            k,
            HandlerKind::Entrypoint
                | HandlerKind::PragmaEntrypoint
                | HandlerKind::PrepareReadContent
                | HandlerKind::ExecuteReadContent
                | HandlerKind::PrepareWriteContent
                | HandlerKind::CompareContent
                | HandlerKind::ExecuteTransport
        )
    });
    let needs_helper = handler_kinds.contains(&HandlerKind::ExecuteReadContent)
        || handler_kinds.contains(&HandlerKind::ExecuteTransport);

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
    // Note: serde_json Cargo dep is added when pragma helpers are present,
    // but pragma helpers use fully-qualified `serde_json::Value` paths
    // so no `use` import is needed here.

    // ── Op enum (Raw — optional data payload for literal-source nodes) ──
    items.push(build_op_enum_raw(handler_kinds));

    // ── impl Executable for Op (Raw — match indentation) ──
    items.push(build_executable_impl_raw(handler_kinds));

    // ── Pragma helpers (Raw) ──
    if requires_pragma_helpers(handler_kinds) {
        let mut pragma_text = String::new();
        emit_pragma_helpers(&mut pragma_text);
        let trimmed = pragma_text.trim().to_string();
        // Raw because pragma helpers are emitted as a preformatted Rust text block.
        items.push(Item::Raw(trimmed));
    }

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

    // Edges.
    for edge in &dag.edges {
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
        for port in &node.inputs {
            if port.name.0.starts_with("res:") || port.name.0.starts_with("__") {
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
    writeln!(
        text,
        "    let args: Vec<String> = std::env::args().collect();"
    )
    .unwrap();
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
            "    let result = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, None);"
        )
        .unwrap();
    } else {
        writeln!(text, "    let result = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks));").unwrap();
    }
    writeln!(text, "    match result {{").unwrap();
    writeln!(text, "        Ok(log) => {{").unwrap();
    writeln!(
        text,
        r#"            eprintln!("execution completed: {{}} nodes executed", log.entries.len());"#
    )
    .unwrap();
    writeln!(text, "            for entry in &log.entries {{").unwrap();
    writeln!(
        text,
        r#"                eprintln!("  [{{}}] intercepted={{}}", entry.node_id, entry.was_intercepted);"#
    )
    .unwrap();
    writeln!(text, "            }}").unwrap();
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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{
        CallableKind, LoweredOp, ObligationCategory, PrimitiveLiteral, PrimitiveOpKind,
    };
    use gunbc_ir::{Edge, Node, Port};

    fn sample_makegen_dag() -> Dag<LoweredOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![Port::scalar("registry", "ToolRegistry")],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Fn,
                name: "load_registry".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_node(Node::opaque(
            "render_makefile",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("return", "String")],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Fn,
                name: "render_makefile".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_edge(Edge::new(
            "load_registry",
            "registry",
            "render_makefile",
            "registry",
        ));
        dag
    }

    #[test]
    fn emit_exec_runtime_produces_two_files() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[1].path, "Cargo.toml");
    }

    #[test]
    fn emitted_main_rs_contains_op_enum() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let main_rs = &files[0].content;
        assert!(main_rs.contains("enum Op {"), "should contain Op enum");
        assert!(
            main_rs.contains("LoadRegistry"),
            "should have LoadRegistry variant"
        );
        assert!(
            main_rs.contains("RenderMakefile"),
            "should have RenderMakefile variant"
        );
    }

    #[test]
    fn emitted_main_rs_contains_executable_impl() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("impl Executable for Op"),
            "should have Executable impl"
        );
        assert!(
            main_rs.contains("fn execute("),
            "should have execute method"
        );
    }

    #[test]
    fn emitted_main_rs_contains_handler_bodies() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("fn execute_load_registry"),
            "should emit load_registry handler"
        );
        assert!(
            main_rs.contains("fn execute_render_makefile"),
            "should emit render_makefile handler"
        );
    }

    #[test]
    fn emitted_main_rs_contains_build_dag() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("fn build_dag()"),
            "should contain build_dag"
        );
        assert!(main_rs.contains("Dag::new()"), "should construct Dag");
        assert!(main_rs.contains("dag.add_node"), "should add nodes");
        assert!(main_rs.contains("dag.add_edge"), "should add edges");
    }

    #[test]
    fn emitted_main_rs_contains_main_fn() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("fn main()"),
            "should contain main function"
        );
        assert!(
            main_rs.contains("execute_with_mode_and_inputs"),
            "should call executor"
        );
    }

    #[test]
    fn emitted_cargo_toml_has_correct_deps() {
        let dag = sample_makegen_dag();
        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let toml = &files[1].content;
        assert!(
            toml.contains(r#"name = "tools-makegen""#),
            "crate name should be sanitized"
        );
        assert!(toml.contains("gunbc-ir"), "should depend on gunbc-ir");
        assert!(toml.contains("gunbc-exec"), "should depend on gunbc-exec");
        // sample_makegen_dag has no pragma helpers → no serde_json
        assert!(
            !toml.contains("serde_json"),
            "should not depend on serde_json (no pragma helpers)"
        );
        // sample_makegen_dag has no transport handlers → no gunbc-lib-transport
        assert!(
            !toml.contains("gunbc-lib-transport"),
            "should not depend on gunbc-lib-transport (no transport handlers)"
        );
    }

    #[test]
    fn emitted_cargo_toml_uses_output_relative_workspace_dependency_paths() {
        let dag = sample_makegen_dag();
        let layout = WorkspaceLayout::from_env_manifest_dir().expect("resolve workspace layout");
        let out_dir = layout
            .workspace_root
            .join("target")
            .join("exec_runtime_nested")
            .join("a")
            .join("b")
            .join("tools-makegen");
        let files = emit_exec_runtime_with_output_dir(&dag, "tools.makegen", Some(&out_dir))
            .expect("should succeed");
        let toml = &files[1].content;
        let ir_path = normalize_dep_path(&layout.relative_path(
            &out_dir,
            layout.crate_dir("gunbc-ir").expect("gunbc-ir crate"),
        ));
        let exec_path = normalize_dep_path(&layout.relative_path(
            &out_dir,
            layout.crate_dir("gunbc-exec").expect("gunbc-exec crate"),
        ));
        assert!(
            toml.contains(&format!("gunbc-ir = {{ path = \"{ir_path}\" }}")),
            "expected workspace-relative gunbc-ir dependency path, got:\n{toml}"
        );
        assert!(
            toml.contains(&format!("gunbc-exec = {{ path = \"{exec_path}\" }}")),
            "expected workspace-relative gunbc-exec dependency path, got:\n{toml}"
        );
    }

    #[test]
    fn emit_exec_runtime_rejects_unknown_module() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "unknown_op",
            vec![],
            vec![],
            LoweredOp::Callable {
                module: "tools.unknown".to_string(),
                kind: CallableKind::Fn,
                name: "something".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        let err = emit_exec_runtime(&dag, "tools.unknown").expect_err("should fail");
        assert!(
            matches!(err, ExecRuntimeError::UnresolvableNode { .. }),
            "should be UnresolvableNode error"
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
    fn emit_exec_runtime_rejects_unmapped_known_module() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "tools.build::build_all",
            vec![Port::scalar("__deps", "Any")],
            vec![Port::scalar("return", "Json")],
            LoweredOp::Callable {
                module: "tools.build".to_string(),
                kind: CallableKind::Func,
                name: "build_all".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));

        let error =
            emit_exec_runtime(&dag, "tools.build").expect_err("unmapped module should fail");
        assert!(
            matches!(error, ExecRuntimeError::UnresolvableNode { .. }),
            "expected unresolvable node error, got {error:?}"
        );
    }

    #[test]
    fn emit_exec_runtime_full_content_upsert_chain() {
        let mut dag = Dag::new();
        // Build a full makegen content_upsert chain.
        dag.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![Port::scalar("registry", "ToolRegistry")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "load_registry".into(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_node(Node::opaque(
            "render_makefile",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("content", "String")],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Fn,
                name: "render_makefile".into(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_node(Node::opaque(
            "makegen",
            vec![Port::scalar("path", "String")],
            vec![
                Port::scalar("path_out", "String"),
                Port::scalar("written", "Bool"),
            ],
            LoweredOp::Callable {
                module: "tools.makegen".into(),
                kind: CallableKind::Func,
                name: "makegen".into(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_node(Node::opaque(
            "prepare_read_makegen",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::prepare_read_makegen".into(),
                kind: PrimitiveOpKind::IoPrepareFileRead,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_read_makegen",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::execute_read_makegen".into(),
                kind: PrimitiveOpKind::IoExecuteFileRead,
            },
        ));
        dag.add_node(Node::opaque(
            "compare_makegen_content",
            vec![
                Port::scalar("expected_content", "String"),
                Port::scalar("response", "TransportResponse"),
            ],
            vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::compare_makegen_content".into(),
                kind: PrimitiveOpKind::CompareEquality,
            },
        ));
        dag.add_node(Node::opaque(
            "prepare_write_makegen",
            vec![
                Port::scalar("content", "String"),
                Port::scalar("path", "String"),
            ],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::prepare_write_makegen".into(),
                kind: PrimitiveOpKind::IoPrepareFileWrite,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_makegen_transport",
            vec![
                Port::scalar("request", "TransportRequest"),
                Port::scalar("skip", "Bool"),
            ],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "tools.makegen".into(),
                name: "content_upsert::execute_makegen_transport".into(),
                kind: PrimitiveOpKind::IoExecuteFileWrite,
            },
        ));

        // Edges.
        dag.add_edge(Edge::new(
            "load_registry",
            "registry",
            "render_makefile",
            "registry",
        ));
        dag.add_edge(Edge::new(
            "makegen",
            "path_out",
            "prepare_read_makegen",
            "path",
        ));
        dag.add_edge(Edge::new(
            "prepare_read_makegen",
            "request",
            "execute_read_makegen",
            "request",
        ));
        dag.add_edge(Edge::new(
            "render_makefile",
            "content",
            "compare_makegen_content",
            "expected_content",
        ));
        dag.add_edge(Edge::new(
            "execute_read_makegen",
            "response",
            "compare_makegen_content",
            "response",
        ));
        dag.add_edge(Edge::new(
            "render_makefile",
            "content",
            "prepare_write_makegen",
            "content",
        ));
        dag.add_edge(Edge::new(
            "makegen",
            "path_out",
            "prepare_write_makegen",
            "path",
        ));
        dag.add_edge(Edge::new(
            "prepare_write_makegen",
            "request",
            "execute_makegen_transport",
            "request",
        ));
        dag.add_edge(Edge::new(
            "compare_makegen_content",
            "skip",
            "execute_makegen_transport",
            "skip",
        ));

        let files = emit_exec_runtime(&dag, "tools.makegen").expect("should succeed");
        let main_rs = &files[0].content;

        // All handler kinds should be present.
        assert!(main_rs.contains("LoadRegistry"), "missing LoadRegistry");
        assert!(main_rs.contains("RenderMakefile"), "missing RenderMakefile");
        assert!(main_rs.contains("Entrypoint"), "missing Entrypoint");
        assert!(
            main_rs.contains("PrepareReadContent"),
            "missing PrepareReadContent"
        );
        assert!(
            main_rs.contains("ExecuteReadContent"),
            "missing ExecuteReadContent"
        );
        assert!(main_rs.contains("CompareContent"), "missing CompareContent");
        assert!(
            main_rs.contains("PrepareWriteContent"),
            "missing PrepareWriteContent"
        );
        assert!(
            main_rs.contains("ExecuteTransport"),
            "missing ExecuteTransport"
        );

        // DAG construction should reference all 8 nodes.
        let add_node_count = main_rs.matches("dag.add_node").count();
        assert_eq!(
            add_node_count, 8,
            "expected 8 add_node calls, got {add_node_count}"
        );

        // DAG construction should reference all 9 edges.
        let add_edge_count = main_rs.matches("dag.add_edge").count();
        assert_eq!(
            add_edge_count, 9,
            "expected 9 add_edge calls, got {add_edge_count}"
        );

        // Entrypoint: makegen.path is unconnected → should parse from CLI args.
        assert!(
            main_rs.contains("input_mocks"),
            "should set up input mocks for entrypoints"
        );
    }

    #[test]
    fn emit_exec_runtime_supports_pragma_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "tools.pragma::render_clippy_toml",
            vec![
                Port::scalar("directives", "List<PragmaDirective>"),
                Port::scalar("__deps", "Any"),
            ],
            vec![Port::scalar("return", "String")],
            LoweredOp::Callable {
                module: "tools.pragma".into(),
                kind: CallableKind::Fn,
                name: "render_clippy_toml".into(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_node(Node::opaque(
            "prepare_read_pragma",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "tools.pragma".into(),
                name: "content_upsert::prepare_read_pragma".into(),
                kind: PrimitiveOpKind::IoPrepareFileRead,
            },
        ));
        dag.add_node(Node::opaque(
            "tools.pragma::pragma",
            vec![
                Port::scalar("directives", "List<PragmaDirective>"),
                Port::scalar("__deps", "Any"),
            ],
            vec![
                Port::scalar("clippy_written", "Bool"),
                Port::scalar("allowlist_written", "Bool"),
                Port::scalar("policy_written", "Bool"),
            ],
            LoweredOp::Callable {
                module: "tools.pragma".into(),
                kind: CallableKind::Func,
                name: "pragma".into(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));

        let files = emit_exec_runtime(&dag, "tools.pragma").expect("pragma emit should succeed");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("RenderPragmaClippyToml"),
            "should include pragma clippy render handler"
        );
        assert!(
            main_rs.contains("PragmaEntrypoint"),
            "should include pragma entrypoint handler"
        );
        assert!(
            main_rs.contains("parse_pragma_directives"),
            "should emit pragma directives parsing helper"
        );
        assert!(
            !main_rs.contains("set_input(\"tools.pragma::render_clippy_toml\", \"__deps\""),
            "internal __deps inputs should not be surfaced as CLI args"
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
    fn classify_handler_supports_sdlc_and_service_transport_surfaces() {
        let sdlc_callable = LoweredOp::Callable {
            module: "pipelines.sdlc".into(),
            kind: CallableKind::Fn,
            name: "default_repo_owner".into(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        };
        assert_eq!(
            classify_handler(&sdlc_callable),
            Some(HandlerKind::ParamSource)
        );

        let pattern_callable = LoweredOp::Callable {
            module: "std.patterns".into(),
            kind: CallableKind::Pattern,
            name: "file_content_matches".into(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        };
        assert_eq!(
            classify_handler(&pattern_callable),
            Some(HandlerKind::ParamSource)
        );

        let service_prepare = LoweredOp::Callable {
            module: "services.sdlc.control_plane".into(),
            kind: CallableKind::Pattern,
            name: "service_transport::prepare::sdlc.ControlPlane::AcquireStageClaim".into(),
            obligation: ObligationCategory::ServiceTransportPrepare,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
        };
        assert_eq!(
            classify_handler(&service_prepare),
            Some(HandlerKind::ParamSource)
        );
    }

    #[test]
    fn emit_exec_runtime_uses_skipped_fallbacks_for_missing_transport_inputs() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "prepare_read_node",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Primitive {
                module: "std.patterns".into(),
                name: "content_upsert::prepare_read".into(),
                kind: PrimitiveOpKind::IoPrepareFileRead,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_read_node",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Primitive {
                module: "std.patterns".into(),
                name: "content_upsert::execute_read".into(),
                kind: PrimitiveOpKind::IoExecuteFileRead,
            },
        ));

        let files = emit_exec_runtime(&dag, "pipelines.sdlc").expect("emit should succeed");
        let main_rs = &files[0].content;
        assert!(
            main_rs.contains("Value::Skipped"),
            "generated runtime should emit skipped fallbacks for missing transport inputs"
        );
    }
}
