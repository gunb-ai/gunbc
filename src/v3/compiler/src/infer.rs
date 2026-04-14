// Type inference: fills Port.value_type by propagating types
// forward through the DAG.
//
// M0 scope:
//   - ValueNode(literal)  -> output port = literal type
//   - TransformNode       -> output type from target function's
//                            signature (user signature registry OR
//                            primitive table)
//   - BranchNode          -> output type = unified path outputs
//   - LoopNode            -> output type pre-set during lowering
//                            from declared return type; infer
//                            leaves it alone
//   - BindNode            -> no output port to fill; the bound name
//                            reuses its value port via the scope
//                            map set during lowering
//
// On inference failure (not exercised by Tests 1-3 yet): call
// DiagnosticTable::mark_unresolved to nullify the port's type AND
// record the diagnostic atomically. Never throw, never return
// Result<_, TypeError>. Guardrail G5.
//
// Fixpoint loop: the DAG is topologically ordered, but inference
// inside a recursive function body depends on the enclosing
// function's declared return type. We iterate until no port types
// change to make sure transitively-dependent ports settle.

use crate::dag::{Behavior, Dag, FunctionRef, LiteralValue, PortId};
use crate::types::{Prim, TypeShape};

pub fn infer(dag: &mut Dag) {
    loop {
        let mut changed = false;
        let node_count = dag.nodes().len();
        for i in 0..node_count {
            if let Some((port, ty)) = decide(dag, i) {
                if dag.port(port).value_type() != Some(&ty) {
                    dag.set_port_type(port, ty);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
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
            let sig = lookup_signature(dag, &t.target)?;
            if sig.params.len() != t.inputs.len() {
                return None;
            }
            Some((t.output, sig.return_type))
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

/// Resolve a FunctionRef to a signature:
///   1. User function registry (populated during lowering)
///   2. Hardcoded primitive table (M0 placeholder for std/ algebra
///      declarations — migrates in M1)
fn lookup_signature(dag: &Dag, target: &FunctionRef) -> Option<ResolvedSignature> {
    if let Some(sig) = dag.signature(&target.name) {
        return Some(ResolvedSignature {
            params: sig.params.clone(),
            return_type: sig.return_type.clone(),
        });
    }
    primitive_signature(&target.name)
}

struct ResolvedSignature {
    params: Vec<TypeShape>,
    return_type: TypeShape,
}

fn primitive_signature(name: &str) -> Option<ResolvedSignature> {
    let int = || TypeShape::Primitive(Prim::Int);
    let bool_ty = || TypeShape::Primitive(Prim::Bool);
    match name {
        "std::int::add" | "std::int::sub" | "std::int::mul" | "std::int::div" => {
            Some(ResolvedSignature {
                params: vec![int(), int()],
                return_type: int(),
            })
        }
        "std::int::eq" | "std::int::ne" | "std::int::lt" | "std::int::le"
        | "std::int::gt" | "std::int::ge" => Some(ResolvedSignature {
            params: vec![int(), int()],
            return_type: bool_ty(),
        }),
        _ => None,
    }
}
