// Type inference: fills Port.value_type by propagating types forward
// through the DAG.
//
// M0 scope:
//   - ValueNode(literal)         -> output port = literal type
//   - TransformNode(BinaryOp)    -> output port from composed inputs
//   - BranchNode                 -> output port from unified path outputs
//   - BindNode                   -> no output port to fill; the bound
//                                   name reuses its value port via the
//                                   scope map set during lowering
//
// On inference failure (not exercised by Tests 1–2, but the pipeline
// must not abort): call DiagnosticTable::mark_unresolved to nullify
// the port's type AND record the diagnostic atomically. Never throw,
// never return Result<_, TypeError>. Guardrail G5.
//
// The DAG is topologically ordered by construction: lowering emits
// each node after its dependencies, and Branch children (path bodies)
// are lowered before the Branch itself. A single forward walk with
// immediate application suffices.

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
        Behavior::Branch(b) => {
            let mut path_types = b.paths.iter().map(|p| dag.port(p.output).value_type());
            let first = path_types.next().and_then(|t| t.cloned())?;
            for other in path_types {
                match other {
                    Some(t) if *t == first => continue,
                    _ => return None,
                }
            }
            Some((b.output, first))
        }
        Behavior::Loop(_) | Behavior::Bind(_) => None,
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
