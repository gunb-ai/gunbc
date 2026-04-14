// SurfaceAst -> Dag lowering.
//
// Walks the surface tree and builds the L1 behavior graph. A
// HashMap<String, PortId> tracks let-binding names in scope so that
// a later reference to a name resolves to the same port.
//
// Surface -> L1 map:
//   IntLit             -> Value
//   Var (local)        -> scope lookup (no new node)
//   Call               -> Transform { target: FunctionRef, inputs }
//                         (operators like `+` pre-resolved to
//                         "std::int::add" etc. by parse.rs)
//   If/then/else       -> Branch with 2 Paths
//   Fn item            -> Bind with non-empty params field
//                         + Loop wrapper when body is recursive
//   Let item           -> Bind with empty params field

use std::collections::HashMap;

use crate::dag::{
    Behavior, BindNode, Bound, BranchNode, Dag, FunctionRef, LiteralValue, LoopNode, NodeId, Path,
    PortId, Signature, TransformNode, ValueNode,
};
use crate::parse::{SurfaceExpr, SurfaceItem, SurfaceModule, SurfaceParam, SurfaceType};
use crate::types::{Prim, TypeShape};

pub fn lower(module: &SurfaceModule) -> Dag {
    let mut dag = Dag::new();
    let mut scope: HashMap<String, PortId> = HashMap::new();
    for item in &module.items {
        lower_item(item, &mut dag, &mut scope);
    }
    dag
}

fn lower_item(item: &SurfaceItem, dag: &mut Dag, scope: &mut HashMap<String, PortId>) {
    match item {
        SurfaceItem::Let { name, expr } => {
            let value_port = lower_expr(expr, dag, scope);
            let bind_id = dag.alloc_node_id();
            dag.push_node(Behavior::Bind(BindNode {
                id: bind_id,
                name: name.clone(),
                value: value_port,
                params: Vec::new(),
                scope: None,
            }));
            scope.insert(name.clone(), value_port);
        }
        SurfaceItem::Fn {
            name,
            params,
            return_type,
            body,
        } => {
            lower_fn_item(name, params, return_type, body, dag, scope);
        }
    }
}

fn lower_fn_item(
    name: &str,
    params: &[SurfaceParam],
    return_type: &SurfaceType,
    body: &SurfaceExpr,
    dag: &mut Dag,
    outer_scope: &mut HashMap<String, PortId>,
) {
    // 1. Allocate parameter ports and set their declared types.
    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    let mut param_types: Vec<TypeShape> = Vec::with_capacity(params.len());
    let mut body_scope: HashMap<String, PortId> = outer_scope.clone();
    for param in params {
        let port = dag.alloc_port(None);
        let ty = lower_type(&param.ty);
        dag.set_port_type(port, ty.clone());
        body_scope.insert(param.name.clone(), port);
        param_ports.push(port);
        param_types.push(ty);
    }

    // 2. Register the function's declared signature BEFORE lowering
    //    the body, so recursive Transforms in the body can resolve
    //    their own function's return type without a cycle.
    let return_ty = lower_type(return_type);
    dag.register_signature(
        name,
        Signature {
            params: param_types.clone(),
            return_type: return_ty.clone(),
        },
    );

    // 3. Lower the body in the extended scope.
    let body_return_port = lower_expr(body, dag, &body_scope);
    let body_root = producer_of(dag, body_return_port);

    // 4. If the body is recursive, wrap it in a Loop with the first
    //    parameter as the bound count (the M0 pattern: structurally
    //    smaller argument on each recursive call).
    let value_port = if is_recursive(body, name) && !param_ports.is_empty() {
        let loop_id = dag.alloc_node_id();
        let loop_output = dag.alloc_port(Some(loop_id));
        dag.set_port_type(loop_output, return_ty.clone());
        dag.push_node(Behavior::Loop(LoopNode {
            id: loop_id,
            source: param_ports[0],
            init: param_ports[0],
            body: body_root,
            bound: Bound {
                count: param_ports[0],
            },
            output: loop_output,
        }));
        loop_output
    } else {
        body_return_port
    };

    // 5. Create the Bind for the function. The value port's type is
    //    the declared return type — pre-set so inference for call
    //    sites can trust it immediately.
    dag.set_port_type(value_port, return_ty);
    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: name.to_string(),
        value: value_port,
        params: param_ports,
        scope: None,
    }));
    outer_scope.insert(name.to_string(), value_port);
}

fn lower_type(ty: &SurfaceType) -> TypeShape {
    match ty {
        SurfaceType::Named(name) => match name.as_str() {
            "Int" => TypeShape::Primitive(Prim::Int),
            "Bool" => TypeShape::Primitive(Prim::Bool),
            "String" => TypeShape::Primitive(Prim::String),
            _ => TypeShape::Primitive(Prim::Int),
        },
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
        SurfaceExpr::Call { target, args, .. } => {
            let input_ports: Vec<PortId> =
                args.iter().map(|a| lower_expr(a, dag, scope)).collect();
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: FunctionRef::new(target.clone()),
                inputs: input_ports,
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

fn producer_of(dag: &Dag, port: PortId) -> NodeId {
    dag.port(port)
        .produced_by
        .expect("lowered expressions always have a producing node")
}

fn is_recursive(expr: &SurfaceExpr, self_name: &str) -> bool {
    match expr {
        SurfaceExpr::IntLit { .. } | SurfaceExpr::Var { .. } => false,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                return true;
            }
            args.iter().any(|a| is_recursive(a, self_name))
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            is_recursive(cond, self_name)
                || is_recursive(then_branch, self_name)
                || is_recursive(else_branch, self_name)
        }
    }
}
