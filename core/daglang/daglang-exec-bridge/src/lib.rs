use std::collections::HashMap;

use daglang_lower::{classify_runtime_op, CollectionOpKind, LoweredOp, RuntimeOpId};
use gunbc_exec::{
    execute_with_mode_and_inputs, BoundaryMocks, ExecError, Executable, ExecutionLog,
    ExecutionMode, OutputMap,
};
use gunbc_ir::transport::{FileOp, FileRequest, FileResponse, TransportRequest, TransportResponse};
use gunbc_ir::{Dag, Node, Value};
use gunbc_lib_transport::executor::execute_transport;
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedOp {
    LoadRegistry,
    FsEnv,
    RenderMakefile,
    MakegenEntrypoint,
    PrepareReadContent,
    ExecuteReadContent,
    PrepareWriteContent,
    CompareContent,
    ExecuteTransport,
    CollectionNode(CollectionOpKind),
}

fn resolved_op_from_runtime_id(op_id: RuntimeOpId) -> ResolvedOp {
    match op_id {
        RuntimeOpId::MakegenLoadRegistry => ResolvedOp::LoadRegistry,
        RuntimeOpId::MakegenFsEnv => ResolvedOp::FsEnv,
        RuntimeOpId::MakegenRenderMakefile => ResolvedOp::RenderMakefile,
        RuntimeOpId::MakegenEntrypoint => ResolvedOp::MakegenEntrypoint,
        RuntimeOpId::MakegenPrepareReadContent => ResolvedOp::PrepareReadContent,
        RuntimeOpId::MakegenExecuteReadContent => ResolvedOp::ExecuteReadContent,
        RuntimeOpId::MakegenPrepareWriteContent => ResolvedOp::PrepareWriteContent,
        RuntimeOpId::MakegenCompareContent => ResolvedOp::CompareContent,
        RuntimeOpId::MakegenExecuteTransport => ResolvedOp::ExecuteTransport,
        RuntimeOpId::Collection(kind) => ResolvedOp::CollectionNode(kind),
    }
}

impl Executable for ResolvedOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            Self::LoadRegistry => execute_load_registry(inputs),
            Self::FsEnv => execute_fs_env(inputs),
            Self::RenderMakefile => execute_render_makefile(inputs),
            Self::MakegenEntrypoint => execute_finalize_makegen(inputs),
            Self::PrepareReadContent => execute_prepare_read_content(inputs),
            Self::ExecuteReadContent => run_read_content_node(inputs),
            Self::PrepareWriteContent => execute_prepare_write_content(inputs),
            Self::CompareContent => execute_compare_content(inputs),
            Self::ExecuteTransport => run_transport_node(inputs),
            Self::CollectionNode(_) => execute_collection_node(inputs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveDagError {
    pub node_id: String,
    pub reason: String,
}

impl std::fmt::Display for ResolveDagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resolve error at `{}`: {}", self.node_id, self.reason)
    }
}

pub fn resolve_lowered_dag(dag: &Dag<LoweredOp>) -> Result<Dag<ResolvedOp>, ResolveDagError> {
    let mut resolved = Dag::new();
    for node in &dag.nodes {
        let resolved_op = resolve_lowered_node(node)?;
        resolved.add_node(node.clone().map_ops(&mut |_| resolved_op.clone()));
    }
    resolved.edges = dag.edges.clone();
    Ok(resolved)
}

pub fn execute_resolved_dag(
    dag: &Dag<ResolvedOp>,
    mode: ExecutionMode,
    input_mocks: Option<&BoundaryMocks>,
) -> Result<ExecutionLog, ExecError> {
    execute_with_mode_and_inputs(dag, mode, input_mocks)
}

pub fn makegen_entrypoint_mocks(output_path: &str) -> BoundaryMocks {
    let mut input_mocks = BoundaryMocks::new();
    input_mocks.set_input(
        "prepare_read_makegen",
        "path",
        Value::Str(output_path.to_string()),
    );
    input_mocks.set_input(
        "prepare_write_makegen",
        "path",
        Value::Str(output_path.to_string()),
    );
    input_mocks
}

pub fn makegen_dry_run_transport_mocks(output_path: &str) -> BoundaryMocks {
    let mut dry_run_mocks = BoundaryMocks::new();
    dry_run_mocks.set_value(
        "fs_env",
        "FilesystemHandle",
        Value::Str("filesystem://dry-run".to_string()),
    );
    dry_run_mocks.set_value(
        "execute_read_makegen",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok(
            output_path.to_string(),
            "<dry-run>",
        ))),
    );
    dry_run_mocks.set_value(
        "execute_makegen_transport",
        "makegen_response",
        Value::Skipped,
    );
    dry_run_mocks
}

pub fn makegen_check_mode_transport_mocks(output_path: &str) -> BoundaryMocks {
    let mut check_mode_mocks = BoundaryMocks::new();
    check_mode_mocks.set_value(
        "fs_env",
        "FilesystemHandle",
        Value::Str("filesystem://check-mode".to_string()),
    );
    let existing_content = read_existing_content(output_path);
    check_mode_mocks.set_value(
        "execute_read_makegen",
        "response",
        Value::Response(TransportResponse::File(FileResponse::read_ok(
            output_path.to_string(),
            existing_content,
        ))),
    );
    check_mode_mocks.set_value(
        "execute_makegen_transport",
        "makegen_response",
        Value::Skipped,
    );
    check_mode_mocks
}

fn read_existing_content(output_path: &str) -> String {
    let response = execute_file_request(&FileRequest::read(output_path));
    if response.success {
        return response.content.unwrap_or_default();
    }
    String::new()
}

fn execute_load_registry(
    _inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    OutputMap::new()
        .json(
            "registry",
            json!({
                "tools": [
                    {
                        "name": "makegen",
                        "command": "cargo run -p gunbc-dag --bin gunbc-makegen"
                    }
                ]
            }),
        )
        .ok()
}

fn execute_fs_env(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    OutputMap::new()
        .str("FilesystemHandle", "filesystem://workspace")
        .ok()
}

fn execute_render_makefile(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let registry = inputs
        .get("registry")
        .and_then(Value::as_json)
        .ok_or_else(|| ExecError::new("missing required input `registry`".to_string()))?;
    let tools = registry
        .get("tools")
        .and_then(|value| value.as_array())
        .ok_or_else(|| ExecError::new("registry must contain `tools` array".to_string()))?;

    let mut targets = Vec::new();
    let mut lines = Vec::new();
    for tool in tools {
        let name = tool
            .get("name")
            .and_then(|value| value.as_str())
            .ok_or_else(|| ExecError::new("registry tool entry missing `name`".to_string()))?;
        let command = tool
            .get("command")
            .and_then(|value| value.as_str())
            .ok_or_else(|| ExecError::new("registry tool entry missing `command`".to_string()))?;
        targets.push(name.to_string());
        lines.push(format!("{name}:\n\t{command}"));
    }

    let content = format!(
        "# Generated by daglang\n.PHONY: {}\n\n{}\n",
        targets.join(" "),
        lines.join("\n\n")
    );
    OutputMap::new().str("return", content).ok()
}

fn execute_prepare_read_content(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let path = inputs
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required input `path`".to_string()))?;
    OutputMap::new()
        .request("request", TransportRequest::File(FileRequest::read(path)))
        .ok()
}

fn run_read_content_node(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let request = inputs
        .get("request")
        .and_then(Value::as_request)
        .ok_or_else(|| ExecError::new("missing required input `request`".to_string()))?;
    match request {
        TransportRequest::File(file_request) => OutputMap::new()
            .response(
                "response",
                TransportResponse::File(execute_file_request(&file_request)),
            )
            .ok(),
        _ => Err(ExecError::new(
            "only file transport requests are supported by daglang-cli".to_string(),
        )),
    }
}

fn execute_prepare_write_content(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let path = inputs
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required input `path`".to_string()))?;
    let content = inputs
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required input `content`".to_string()))?;
    OutputMap::new()
        .request(
            "request",
            TransportRequest::File(FileRequest::write(path, content)),
        )
        .ok()
}

fn execute_compare_content(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let expected = inputs
        .get("expected_content")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::new("missing required input `expected_content`".to_string()))?;
    let actual = match inputs.get("response") {
        Some(Value::Response(TransportResponse::File(file_response))) if file_response.success => {
            file_response.content.clone().unwrap_or_default()
        }
        _ => String::new(),
    };
    let fresh = actual == expected;
    OutputMap::new()
        .bool("fresh", fresh)
        .bool("skip", fresh)
        .ok()
}

fn run_transport_node(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let skip = inputs.get("skip").and_then(Value::as_bool).unwrap_or(false);
    if skip {
        return OutputMap::new()
            .value("makegen_response", Value::Skipped)
            .ok();
    }

    let request = inputs
        .get("request")
        .and_then(Value::as_request)
        .ok_or_else(|| ExecError::new("missing required input `request`".to_string()))?;
    match request {
        TransportRequest::File(file_request) => {
            let response = execute_file_request(&file_request);
            if file_request.operation == FileOp::Write && !response.success {
                let error = response
                    .error
                    .clone()
                    .unwrap_or_else(|| "unknown write failure".to_string());
                return Err(ExecError::new(format!(
                    "failed to write `{}`: {error}",
                    file_request.path
                )));
            }
            OutputMap::new()
                .response("makegen_response", TransportResponse::File(response))
                .ok()
        }
        _ => Err(ExecError::new(
            "only file transport requests are supported by daglang-cli".to_string(),
        )),
    }
}

fn execute_collection_node(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Collection execution is currently a structural passthrough scaffold.
    // Lowering emits collection nodes for progress/parity visibility; runtime
    // semantics stay unchanged until dedicated collection executors land.
    let items = inputs
        .get("items")
        .cloned()
        .ok_or_else(|| ExecError::new("missing required input `items`".to_string()))?;
    OutputMap::new().value("items", items).ok()
}

fn execute_file_request(request: &FileRequest) -> FileResponse {
    match execute_transport(&TransportRequest::File(request.clone())) {
        Ok(TransportResponse::File(response)) => response,
        Ok(_other) => FileResponse::error(
            request.path.clone(),
            request.operation,
            "transport executor returned non-file response for file request",
        ),
        Err(error) => {
            FileResponse::error(request.path.clone(), request.operation, error.to_string())
        }
    }
}

fn execute_finalize_makegen(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let written = inputs
        .get("__deps")
        .and_then(Value::as_list)
        .map(|deps| {
            deps.iter().any(|value| {
                matches!(
                    value,
                    Value::Response(TransportResponse::File(response))
                        if response.operation == FileOp::Write && response.success
                )
            })
        })
        .unwrap_or(false);
    OutputMap::new().bool("written", written).ok()
}

fn resolve_lowered_node(node: &Node<LoweredOp>) -> Result<ResolvedOp, ResolveDagError> {
    let node_id = node.id.0.clone();
    match &node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => resolve_lowered_op(&node_id, op),
        gunbc_ir::node::NodeBody::SubDag(_) => Err(ResolveDagError {
            node_id,
            reason: "subdag nodes are not supported by daglang-cli resolver".to_string(),
        }),
    }
}

fn resolve_lowered_op(node_id: &str, op: &LoweredOp) -> Result<ResolvedOp, ResolveDagError> {
    classify_runtime_op(op)
        .map(resolved_op_from_runtime_id)
        .ok_or_else(|| match op {
            LoweredOp::Pipeline { module, name, .. } => ResolveDagError {
                node_id: node_id.to_string(),
                reason: format!("unsupported pipeline `{module}.{name}` in execution resolver"),
            },
            LoweredOp::Collection { module, callable, kind } => ResolveDagError {
                node_id: node_id.to_string(),
                reason: format!(
                    "unsupported collection node `{module}.{callable}` kind={kind:?} in execution resolver"
                ),
            },
            LoweredOp::Callable { module, name, .. } => ResolveDagError {
                node_id: node_id.to_string(),
                reason: format!(
                    "unsupported callable `{module}.{name}` (missing runtime op classification)"
                ),
            },
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{CallableKind, ObligationCategory};

    #[test]
    fn resolve_lowered_dag_maps_known_makegen_callables() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "load_registry",
            vec![],
            vec![],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Fn,
                name: "load_registry".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        let resolved = resolve_lowered_dag(&dag).expect("makegen callable should resolve");
        let Some(node) = resolved.nodes.first() else {
            panic!("expected one node");
        };
        match &node.body {
            gunbc_ir::node::NodeBody::Opaque(ResolvedOp::LoadRegistry) => {}
            _ => panic!("unexpected resolved op"),
        }
    }

    #[test]
    fn resolve_lowered_dag_rejects_unknown_module() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "callable::external",
            vec![],
            vec![],
            LoweredOp::Callable {
                module: "tools.external".to_string(),
                kind: CallableKind::Fn,
                name: "run".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        let error = resolve_lowered_dag(&dag).expect_err("unsupported module should fail");
        assert!(error
            .reason
            .contains("unsupported callable `tools.external.run`"));
    }

    #[test]
    fn resolve_lowered_dag_maps_collection_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "sample.collections::run::MapNode_0",
            vec![],
            vec![],
            LoweredOp::Collection {
                module: "sample.collections".to_string(),
                callable: "sample.collections::run".to_string(),
                kind: CollectionOpKind::Map,
            },
        ));
        let resolved = resolve_lowered_dag(&dag).expect("collection node should resolve");
        let Some(node) = resolved.nodes.first() else {
            panic!("expected one node");
        };
        match &node.body {
            gunbc_ir::node::NodeBody::Opaque(ResolvedOp::CollectionNode(CollectionOpKind::Map)) => {
            }
            _ => panic!("unexpected resolved op"),
        }
    }

    #[test]
    fn collection_resolved_op_passes_items_through() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "items".to_string(),
            Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ]),
        );
        let outputs = ResolvedOp::CollectionNode(CollectionOpKind::Filter)
            .execute(inputs)
            .expect("collection node should execute");
        assert_eq!(
            outputs.get("items"),
            Some(&Value::List(vec![
                Value::Str("a".to_string()),
                Value::Str("b".to_string()),
            ]))
        );
    }
}
