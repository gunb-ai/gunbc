use std::collections::HashMap;

use daglang_lower::LoweredOp;
use gunbc_exec::{
    execute_with_mode_and_inputs, BoundaryMocks, ExecError, Executable, ExecutionLog,
    ExecutionMode, OutputMap,
};
use gunbc_ir::transport::{
    FileOp, FileRequest, FileResponse, TransportRequest, TransportResponse,
};
use gunbc_ir::{Dag, Node, Value};
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
}

#[derive(Debug, Clone)]
struct OpRegistry {
    makegen_callables: HashMap<&'static str, ResolvedOp>,
}

impl OpRegistry {
    fn makegen() -> Self {
        let mut makegen_callables = HashMap::new();
        makegen_callables.insert("load_registry", ResolvedOp::LoadRegistry);
        makegen_callables.insert("fs_env", ResolvedOp::FsEnv);
        makegen_callables.insert("render_makefile", ResolvedOp::RenderMakefile);
        makegen_callables.insert("makegen", ResolvedOp::MakegenEntrypoint);
        makegen_callables.insert(
            "content_upsert::prepare_read_makegen",
            ResolvedOp::PrepareReadContent,
        );
        makegen_callables.insert(
            "content_upsert::execute_read_makegen",
            ResolvedOp::ExecuteReadContent,
        );
        makegen_callables.insert(
            "content_upsert::prepare_write_makegen",
            ResolvedOp::PrepareWriteContent,
        );
        makegen_callables.insert(
            "content_upsert::compare_makegen_content",
            ResolvedOp::CompareContent,
        );
        makegen_callables.insert(
            "content_upsert::execute_makegen_transport",
            ResolvedOp::ExecuteTransport,
        );
        Self { makegen_callables }
    }

    fn resolve_callable(&self, module: &str, name: &str) -> Option<ResolvedOp> {
        if module != "tools.makegen" {
            return None;
        }
        self.makegen_callables.get(name).cloned()
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
            Self::ExecuteReadContent => execute_execute_read_content(inputs),
            Self::PrepareWriteContent => execute_prepare_write_content(inputs),
            Self::CompareContent => execute_compare_content(inputs),
            Self::ExecuteTransport => execute_execute_transport(inputs),
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
    let registry = OpRegistry::makegen();
    let mut resolved = Dag::new();
    for node in &dag.nodes {
        let resolved_op = resolve_lowered_node(node, &registry)?;
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
    let existing_content = std::fs::read(output_path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
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

fn execute_load_registry(_inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
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

fn execute_execute_read_content(
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
        .request("request", TransportRequest::File(FileRequest::write(path, content)))
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

fn execute_execute_transport(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
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

fn execute_file_request(request: &FileRequest) -> FileResponse {
    match request.operation {
        FileOp::Read => match std::fs::read_to_string(&request.path) {
            Ok(content) => FileResponse::read_ok(request.path.clone(), content),
            Err(error) => FileResponse::error(request.path.clone(), FileOp::Read, error.to_string()),
        },
        FileOp::Write => {
            let write_result = (|| -> Result<(), std::io::Error> {
                if request.create_parents {
                    if let Some(parent) = std::path::Path::new(&request.path).parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                std::fs::write(&request.path, request.content.clone().unwrap_or_default())?;
                Ok(())
            })();
            match write_result {
                Ok(()) => FileResponse::written(request.path.clone()),
                Err(error) => FileResponse::error(request.path.clone(), FileOp::Write, error.to_string()),
            }
        }
        _ => FileResponse::error(
            request.path.clone(),
            request.operation,
            "unsupported file operation",
        ),
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

fn resolve_lowered_node(
    node: &Node<LoweredOp>,
    registry: &OpRegistry,
) -> Result<ResolvedOp, ResolveDagError> {
    let node_id = node.id.0.clone();
    match &node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => resolve_lowered_op(&node_id, op, registry),
        gunbc_ir::node::NodeBody::SubDag(_) => Err(ResolveDagError {
            node_id,
            reason: "subdag nodes are not supported by daglang-cli resolver".to_string(),
        }),
    }
}

fn resolve_lowered_op(
    node_id: &str,
    op: &LoweredOp,
    registry: &OpRegistry,
) -> Result<ResolvedOp, ResolveDagError> {
    match op {
        LoweredOp::Pipeline { module, name, .. } => Err(ResolveDagError {
            node_id: node_id.to_string(),
            reason: format!(
                "unsupported pipeline `{module}.{name}` in execution resolver"
            ),
        }),
        LoweredOp::Callable { module, name, .. } => {
            if module != "tools.makegen" {
                return Err(ResolveDagError {
                    node_id: node_id.to_string(),
                    reason: format!("unsupported callable module `{module}`"),
                });
            }
            registry.resolve_callable(module, name).ok_or_else(|| ResolveDagError {
                node_id: node_id.to_string(),
                reason: format!("unsupported callable `tools.makegen.{name}`"),
            })
        }
    }
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
            },
        ));
        let error = resolve_lowered_dag(&dag).expect_err("unsupported module should fail");
        assert!(error.reason.contains("unsupported callable module"));
    }
}
