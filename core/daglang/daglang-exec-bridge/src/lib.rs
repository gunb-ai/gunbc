use std::collections::HashMap;

use daglang_lower::LoweredOp;
use gunbc_exec::{
    execute_with_mode_and_inputs, BoundaryMocks, ExecError, Executable, ExecutionLog,
    ExecutionMode,
};
use gunbc_ir::{Dag, Node, Value};

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

impl Executable for ResolvedOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        Err(ExecError::new(format!(
            "execution for `{self:?}` is not wired in daglang-exec-bridge yet"
        )))
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
        "execute_read_makegen",
        "response",
        Value::Str(format!("dry-run-read:{output_path}")),
    );
    dry_run_mocks.set_value("execute_makegen_transport", "makegen_response", Value::Skipped);
    dry_run_mocks
}

pub fn makegen_check_mode_transport_mocks(output_path: &str) -> BoundaryMocks {
    let mut check_mode_mocks = BoundaryMocks::new();
    check_mode_mocks.set_value(
        "execute_read_makegen",
        "response",
        Value::Str(format!("check-mode-read:{output_path}")),
    );
    check_mode_mocks.set_value("execute_makegen_transport", "makegen_response", Value::Skipped);
    check_mode_mocks
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
            reason: format!("unsupported pipeline `{module}.{name}` in execution resolver"),
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
