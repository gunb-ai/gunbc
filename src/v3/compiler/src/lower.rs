// SurfaceAst -> Dag lowering.
//
// Walks the surface tree and builds the L1 behavior graph: Value for
// literals, Transform(BinaryOp) for operators, Bind for let names.
// A HashMap<String, PortId> tracks let-binding names in scope so that
// a later reference to a name resolves to the same port.

use std::collections::HashMap;

use crate::dag::{
    Behavior, BinOp, BindNode, Dag, LiteralValue, PortId, TransformNode, TransformRule, ValueNode,
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
        SurfaceStmt::Let { name, expr, .. } => {
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
            .expect("M0.1 requires let-before-use; forward references are Test 2 work"),
        SurfaceExpr::BinOp { op, lhs, rhs, .. } => {
            let lhs_port = lower_expr(lhs, dag, scope);
            let rhs_port = lower_expr(rhs, dag, scope);
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            let rule = match op {
                SurfaceBinOp::Add => TransformRule::BinaryOp(BinOp::Add),
            };
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                rule,
                inputs: vec![lhs_port, rhs_port],
                output,
            }));
            output
        }
    }
}
