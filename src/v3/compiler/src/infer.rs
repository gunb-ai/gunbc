// Type inference: fills Port.value_type by propagating types forward
// through the DAG.
//
// M0.1 scope:
//   - ValueNode(Int literal)     -> output port gets TypeShape::Primitive(Int)
//   - TransformNode(BinaryOp)    -> output port type is composed from
//                                   input port types (both Int -> Int)
//   - BindNode                   -> no output port to fill; the bound
//                                   name reuses its value port by
//                                   reference in the scope map
//
// On inference failure (not exercised by Test 1, but the pipeline
// must not abort): call DiagnosticTable::mark_unresolved to nullify
// the port's type AND record the diagnostic atomically. Never throw,
// never return Result<_, TypeError>. Guardrail G5.
//
// The DAG is topologically ordered by construction (lowering emits
// each node after its dependencies), so a single forward walk with
// immediate application suffices — a Transform's inputs are always
// resolved by the time the Transform itself is visited.

use crate::dag::{Behavior, BinOp, Dag, LiteralValue, PortId, TransformRule};
use crate::types::{Prim, TypeShape};

pub fn infer(dag: &mut Dag) {
    let node_count = dag.nodes().len();
    for i in 0..node_count {
        if let Some((port, ty)) = decide(dag, i) {
            dag.set_port_type(port, ty);
        }
    }
}

fn decide(dag: &Dag, index: usize) -> Option<(PortId, TypeShape)> {
    match &dag.nodes()[index] {
        Behavior::Value(v) => {
            let ty = match &v.data {
                LiteralValue::Int(_) => TypeShape::Primitive(Prim::Int),
                LiteralValue::Bool(_) => TypeShape::Primitive(Prim::Bool),
                LiteralValue::String(_) => TypeShape::Primitive(Prim::String),
            };
            Some((v.output, ty))
        }
        Behavior::Transform(t) => {
            let TransformRule::BinaryOp(op) = &t.rule;
            if t.inputs.len() != 2 {
                return None;
            }
            let lhs_ty = dag.port(t.inputs[0]).value_type().cloned();
            let rhs_ty = dag.port(t.inputs[1]).value_type().cloned();
            infer_binop_result(*op, lhs_ty.as_ref(), rhs_ty.as_ref()).map(|ty| (t.output, ty))
        }
        Behavior::Branch(_) | Behavior::Loop(_) | Behavior::Bind(_) => None,
    }
}

fn infer_binop_result(
    op: BinOp,
    lhs: Option<&TypeShape>,
    rhs: Option<&TypeShape>,
) -> Option<TypeShape> {
    let (lhs, rhs) = (lhs?, rhs?);
    if lhs != rhs {
        return None;
    }
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
            if matches!(lhs, TypeShape::Primitive(Prim::Int)) {
                Some(lhs.clone())
            } else {
                None
            }
        }
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            Some(TypeShape::Primitive(Prim::Bool))
        }
    }
}
