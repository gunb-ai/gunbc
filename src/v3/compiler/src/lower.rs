// SurfaceAst -> Dag lowering.
//
// Walks the surface tree and builds the L1 behavior graph. A
// HashMap<String, PortId> tracks let-binding names in scope so that
// a later reference to a name resolves to the same port.
//
// Surface -> L1 map (M0 scope):
//   IntLit           -> Value
//   Var              -> scope lookup (no new node; reuses producer's port)
//   BinOp            -> Transform(BinaryOp(_))
//   If/then/else     -> Branch with 2 Paths

use std::collections::HashMap;

use crate::dag::{
    Behavior, BinOp, BindNode, BranchNode, Dag, LiteralValue, NodeId, Path, PortId, TransformNode,
    TransformRule, ValueNode,
};
use crate::parse::{SurfaceBinOp, SurfaceExpr, SurfaceModule, SurfaceStmt};

pub fn lower(module: &SurfaceModule) -> Dag {
    let mut dag = Dag::new();
    let mut scope: HashMap<String, PortId> = HashMap::new();
    for stmt in &module.statements {
        lower_stmt(stmt, &mut dag, &mut scope);
    }
    dag
}

fn lower_stmt(stmt: &SurfaceStmt, dag: &mut Dag, scope: &mut HashMap<String, PortId>) {
    match stmt {
        SurfaceStmt::Let { name, expr } => {
            let value_port = lower_expr(expr, dag, scope);
            let bind_id = dag.alloc_node_id();
            dag.push_node(Behavior::Bind(BindNode {
                id: bind_id,
                name: name.clone(),
                value: value_port,
                scope: None,
            }));
            scope.insert(name.clone(), value_port);
        }
    }
}

fn lower_expr(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
) -> PortId {
    match expr {
        SurfaceExpr::IntLit { value, .. } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralValue::Int(*value),
                output,
            }));
            output
        }
        SurfaceExpr::Var { name, .. } => *scope
            .get(name)
            .expect("M0 requires let-before-use; forward references are deferred work"),
        SurfaceExpr::BinOp { op, lhs, rhs, .. } => {
            let lhs_port = lower_expr(lhs, dag, scope);
            let rhs_port = lower_expr(rhs, dag, scope);
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            let rule = TransformRule::BinaryOp(lower_binop(*op));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                rule,
                inputs: vec![lhs_port, rhs_port],
                output,
            }));
            output
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            let cond_port = lower_expr(cond, dag, scope);
            let then_port = lower_expr(then_branch, dag, scope);
            let else_port = lower_expr(else_branch, dag, scope);
            let then_body = producer_of(dag, then_port);
            let else_body = producer_of(dag, else_port);
            let branch_id = dag.alloc_node_id();
            let branch_output = dag.alloc_port(Some(branch_id));
            dag.push_node(Behavior::Branch(BranchNode {
                id: branch_id,
                input: cond_port,
                paths: vec![
                    Path {
                        body: then_body,
                        output: then_port,
                    },
                    Path {
                        body: else_body,
                        output: else_port,
                    },
                ],
                output: branch_output,
            }));
            branch_output
        }
    }
}

fn lower_binop(op: SurfaceBinOp) -> BinOp {
    match op {
        SurfaceBinOp::Add => BinOp::Add,
        SurfaceBinOp::Eq => BinOp::Eq,
        SurfaceBinOp::NotEq => BinOp::Ne,
        SurfaceBinOp::Lt => BinOp::Lt,
        SurfaceBinOp::Le => BinOp::Le,
        SurfaceBinOp::Gt => BinOp::Gt,
        SurfaceBinOp::Ge => BinOp::Ge,
    }
}

fn producer_of(dag: &Dag, port: PortId) -> NodeId {
    dag.port(port)
        .produced_by
        .expect("lowered expressions always have a producing node")
}
