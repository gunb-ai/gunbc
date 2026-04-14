// SurfaceAst -> Dag lowering.
//
// Walks the surface tree and builds the L1 behavior graph. A
// HashMap<String, PortId> tracks let-binding names in scope.
//
// Scope threading is functional by contract: `lower_item` and
// `lower_fn_item` take a `HashMap` by value and return an updated
// `HashMap`. There are no `&mut` references to the scope because
// `&mut` in a recursive parameter position has no .dag analogue;
// the .dag port threads scope as a value through each recursive
// call. The function bodies may use local `mut` bindings to call
// `HashMap::insert`, but the external contract is value-in /
// value-out. See src/v3/ROADMAP.md "Sketch vs Oracle framing"
// for why this particular refactor matters while others don't.
//
// Facts flow forward: every SurfaceExpr has a span, and every
// Behavior we create carries that span as a structural field
// (not a side table). Downstream stages (infer, lenses, emission)
// read the span directly from the node. No reconstruction.
//
// Surface -> L1 map:
//   IntLit             -> Value(LiteralValue::Int)
//   BoolLit            -> Value(LiteralValue::Bool)
//   StringLit          -> Value(LiteralValue::String)
//   Var (local)        -> scope lookup (no new node; reuses producer's port)
//   Var (unresolved)   -> placeholder port + ResolveError diagnostic
//   Call               -> Transform { target: FunctionRef, inputs }
//                         (operators like `+` pre-resolved to
//                         "std::int::add" etc. by parse.rs)
//   If/then/else       -> Branch with 2 Paths
//   Fn item            -> Bind with non-empty params field
//                         + Loop wrapper when body is recursive
//   Let item           -> Bind with empty params field

use std::collections::{HashMap, HashSet};

use crate::dag::{
    Behavior, BindNode, Bound, BranchNode, Dag, FunctionRef, LiteralValue, LoopNode, NodeId, Path,
    PortId, Signature, TransformNode, ValueNode,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::parse::{SurfaceExpr, SurfaceItem, SurfaceModule, SurfaceParam, SurfaceType};
use crate::types::{Prim, TypeShape};

pub fn lower(module: &SurfaceModule) -> Dag {
    let mut dag = Dag::new();
    // M0.11: detect mutual recursion BEFORE lowering any bodies.
    // Any fn in an SCC of size > 1 is rejected at its own Bind
    // level so body lowering never emits call-cycle Transforms —
    // those would otherwise stall the fixpoint on Retry forever
    // and leave the pre-seeded declared return type as a fake
    // Resolved result for call sites.
    let mutually_recursive = compute_mutually_recursive(&module.items);
    let mut scope: HashMap<String, PortId> = HashMap::new();
    for item in &module.items {
        scope = lower_item(item, &mut dag, scope, &mutually_recursive);
    }
    dag
}

fn lower_item(
    item: &SurfaceItem,
    dag: &mut Dag,
    scope: HashMap<String, PortId>,
    mutually_recursive: &HashSet<String>,
) -> HashMap<String, PortId> {
    let mut scope = scope;
    match item {
        SurfaceItem::Let {
            name,
            type_ann,
            expr,
        } => {
            let value_port = lower_expr(expr, dag, &scope);
            if let Some(ty) = type_ann {
                match lower_type(ty) {
                    Ok(declared) => dag.set_port_type(value_port, declared),
                    Err(diag) => dag.mark_unresolved(value_port, diag),
                }
            }
            let bind_id = dag.alloc_node_id();
            let bind_span = match type_ann {
                Some(ty) => ty.span().clone(),
                None => expr_span(expr).clone(),
            };
            dag.push_node(Behavior::Bind(BindNode {
                id: bind_id,
                name: name.clone(),
                value: value_port,
                params: Vec::new(),
                span: bind_span,
            }));
            scope.insert(name.clone(), value_port);
            scope
        }
        SurfaceItem::Fn {
            name,
            params,
            return_type,
            body,
        } => lower_fn_item(
            name,
            params,
            return_type,
            body,
            dag,
            scope,
            mutually_recursive,
        ),
    }
}

fn lower_fn_item(
    name: &str,
    params: &[SurfaceParam],
    return_type: &SurfaceType,
    body: &SurfaceExpr,
    dag: &mut Dag,
    outer_scope: HashMap<String, PortId>,
    mutually_recursive: &HashSet<String>,
) -> HashMap<String, PortId> {
    // 1. Allocate parameter ports and set their declared types.
    //    On unknown type names, mark the param port Unresolved with
    //    a ResolveError and fall through with a sentinel Int type.
    //    This propagates via the cascade logic in infer to every
    //    call site that touches this parameter.
    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    let mut param_types: Vec<TypeShape> = Vec::with_capacity(params.len());
    let mut body_scope: HashMap<String, PortId> = outer_scope.clone();
    for param in params {
        let port = dag.alloc_port(None);
        let ty = match lower_type(&param.ty) {
            Ok(ty) => {
                dag.set_port_type(port, ty.clone());
                ty
            }
            Err(diag) => {
                dag.mark_unresolved(port, diag);
                TypeShape::Primitive(Prim::Int)
            }
        };
        body_scope.insert(param.name.clone(), port);
        param_ports.push(port);
        param_types.push(ty);
    }

    // 2. Register the function's declared signature BEFORE lowering
    //    the body, so recursive Transforms in the body can resolve
    //    their own function's return type without a cycle.
    //    On unknown return type name, we still register (with a
    //    sentinel) so that the structure of the DAG is complete,
    //    but we also allocate a placeholder port to carry the
    //    ResolveError so the fail-closed invariant holds.
    let return_ty = match lower_type(return_type) {
        Ok(ty) => ty,
        Err(diag) => {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(err_port, diag);
            TypeShape::Primitive(Prim::Int)
        }
    };
    dag.register_signature(
        name,
        Signature {
            params: param_types.clone(),
            return_type: return_ty.clone(),
        },
    );

    // 2.5. Mutual recursion check. If this function is part of a
    //      call cycle of size > 1, skip body lowering entirely and
    //      emit a rejection Bind whose value port is Unresolved.
    //      The signature remains registered (so other functions
    //      that call this one can look it up and cascade Fail),
    //      but the Bind.value port's Unresolved state prevents
    //      callers from trusting the signature. This is the
    //      existing Bind-state check from M0.8 in reverse: here,
    //      we preemptively mark the Bind invalid because we know
    //      the body can't be typed without support we haven't
    //      built yet.
    if mutually_recursive.contains(name) {
        let err_port = dag.alloc_port(None);
        let body_span = expr_span(body).clone();
        dag.mark_unresolved(
            err_port,
            Diagnostic::ResolveError {
                name: format!(
                    "function `{name}` is part of a mutual recursion cycle; mutual recursion is not yet supported in v3 M0"
                ),
                span: body_span.clone(),
            },
        );
        let bind_id = dag.alloc_node_id();
        dag.push_node(Behavior::Bind(BindNode {
            id: bind_id,
            name: name.to_string(),
            value: err_port,
            params: param_ports,
            span: body_span,
        }));
        let mut outer_scope = outer_scope;
        outer_scope.insert(name.to_string(), err_port);
        return outer_scope;
    }

    // 3. Lower the body in the extended scope.
    let body_return_port = lower_expr(body, dag, &body_scope);
    let body_root = dag.port(body_return_port).produced_by;
    let body_span = expr_span(body).clone();

    // 4. Decide how to wire up the function's Bind.value port
    //    depending on the shape of the body:
    //
    //    - Not recursive: Bind.value is the body's return port.
    //
    //    - Recursive with zero parameters: non-terminating. Reject
    //      and give the Bind an unresolved placeholder port.
    //
    //    - Recursive, descent provable (first arg to recursive
    //      calls is `first_param - positive_int`): wrap the body in
    //      a Loop node bounded by the first parameter. This is the
    //      M0 partial descent analysis — more sophisticated measures
    //      (lexicographic orderings, multi-param descent) are
    //      deferred.
    //
    //    - Recursive, descent NOT provable: reject and give the
    //      Bind an unresolved placeholder port with an
    //      UnprovenDescent diagnostic. Conservative failure mode:
    //      reject more programs than strictly necessary, never
    //      accept a program that could diverge.
    let value_port = if is_recursive(body, name) {
        if param_ports.is_empty() {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(
                err_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "function `{name}` is recursive but has no parameters; cannot terminate"
                    ),
                    span: body_span.clone(),
                },
            );
            err_port
        } else if !descent_provable(body, name, &params[0].name) {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(
                err_port,
                Diagnostic::ResolveError {
                    name: format!(
                        "cannot prove recursion in `{name}` terminates; expected each recursive call's first argument to be `{param} - <positive int>`",
                        param = &params[0].name,
                    ),
                    span: body_span.clone(),
                },
            );
            err_port
        } else {
            let loop_id = dag.alloc_node_id();
            let loop_output = dag.alloc_port(Some(loop_id));
            dag.set_port_type(loop_output, return_ty.clone());
            let loop_body_node = body_root.unwrap_or(loop_id);
            dag.push_node(Behavior::Loop(LoopNode {
                id: loop_id,
                source: param_ports[0],
                init: param_ports[0],
                body: loop_body_node,
                bound: Bound {
                    count: param_ports[0],
                },
                output: loop_output,
                span: body_span.clone(),
            }));
            loop_output
        }
    } else {
        body_return_port
    };

    // 5. Create the Bind for the function. The value port's type is
    //    the declared return type — pre-set so inference for call
    //    sites can trust it (subject to the Bind-state check in
    //    infer's Transform decide, which catches body/signature
    //    mismatches).
    dag.set_port_type(value_port, return_ty);
    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: name.to_string(),
        value: value_port,
        params: param_ports,
        span: body_span,
    }));
    let mut outer_scope = outer_scope;
    outer_scope.insert(name.to_string(), value_port);
    outer_scope
}

fn lower_type(ty: &SurfaceType) -> Result<TypeShape, Diagnostic> {
    match ty {
        SurfaceType::Named { name, span } => match name.as_str() {
            "Int" => Ok(TypeShape::Primitive(Prim::Int)),
            "Bool" => Ok(TypeShape::Primitive(Prim::Bool)),
            "String" => Ok(TypeShape::Primitive(Prim::String)),
            _ => Err(Diagnostic::ResolveError {
                name: format!("unknown type `{name}`"),
                span: span.clone(),
            }),
        },
    }
}

fn lower_expr(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
) -> PortId {
    match expr {
        SurfaceExpr::IntLit { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralValue::Int(*value),
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::BoolLit { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralValue::Bool(*value),
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::StringLit { value, span } => {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                data: LiteralValue::String(value.clone()),
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::Var { name, span } => match scope.get(name) {
            Some(port) => *port,
            None => {
                // Forward reference: the name isn't in scope yet.
                // Fail-closed: allocate a placeholder port and mark
                // it Unresolved with a ResolveError diagnostic. The
                // pipeline continues so more errors can accumulate.
                let port = dag.alloc_port(None);
                dag.mark_unresolved(
                    port,
                    Diagnostic::ResolveError {
                        name: name.clone(),
                        span: span.clone(),
                    },
                );
                port
            }
        },
        SurfaceExpr::Call { target, args, span } => {
            let input_ports: Vec<PortId> =
                args.iter().map(|a| lower_expr(a, dag, scope)).collect();
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: FunctionRef::new(target.clone()),
                inputs: input_ports,
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => {
            let cond_port = lower_expr(cond, dag, scope);
            let then_port = lower_expr(then_branch, dag, scope);
            let else_port = lower_expr(else_branch, dag, scope);
            let branch_id = dag.alloc_node_id();
            let branch_output = dag.alloc_port(Some(branch_id));
            let then_body = producer_of(dag, then_port).unwrap_or(branch_id);
            let else_body = producer_of(dag, else_port).unwrap_or(branch_id);
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
                span: span.clone(),
            }));
            branch_output
        }
    }
}

fn producer_of(dag: &Dag, port: PortId) -> Option<NodeId> {
    dag.port(port).produced_by
}

fn is_recursive(expr: &SurfaceExpr, self_name: &str) -> bool {
    match expr {
        SurfaceExpr::IntLit { .. }
        | SurfaceExpr::BoolLit { .. }
        | SurfaceExpr::StringLit { .. }
        | SurfaceExpr::Var { .. } => false,
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

/// Partial termination analysis for M0. Walks the body and, for
/// every recursive self-call, verifies that its first argument is
/// of the form `first_param - <positive int>` — i.e. strictly
/// smaller than the first parameter by a constant amount. Returns
/// false if any recursive call fails the check.
///
/// This is intentionally narrow:
/// - Accepts: `f(n - 1)`, `f(n - 2)`, `f(n - 3) + f(n - 1)`
/// - Rejects: `f(n)`, `f(n + 1)`, `f(n * 2)`, `f(m - 1)` where
///   m is not the first param, `f(n - 0)`, `f(1)`, zero-arg calls
///
/// More sophisticated termination analysis (lexicographic orderings
/// on multiple parameters, structural recursion on containers,
/// measure functions that aren't simple subtraction) is M1+ work.
/// The failure mode is conservative: reject more programs than
/// strictly necessary, never accept a program that could diverge.
fn descent_provable(expr: &SurfaceExpr, self_name: &str, first_param: &str) -> bool {
    match expr {
        SurfaceExpr::IntLit { .. }
        | SurfaceExpr::BoolLit { .. }
        | SurfaceExpr::StringLit { .. }
        | SurfaceExpr::Var { .. } => true,
        SurfaceExpr::Call { target, args, .. } => {
            if target == self_name {
                match args.first() {
                    None => false,
                    Some(first_arg) => {
                        if !is_strictly_smaller(first_arg, first_param) {
                            return false;
                        }
                        args.iter()
                            .skip(1)
                            .all(|a| descent_provable(a, self_name, first_param))
                    }
                }
            } else {
                args.iter().all(|a| descent_provable(a, self_name, first_param))
            }
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            descent_provable(cond, self_name, first_param)
                && descent_provable(then_branch, self_name, first_param)
                && descent_provable(else_branch, self_name, first_param)
        }
    }
}

/// Returns true iff `expr` is syntactically `first_param - k` for
/// some positive integer constant `k`. Matches the form emitted by
/// parse.rs for `n - 1` (SurfaceExpr::Call with target
/// "std::int::sub" and two args).
fn is_strictly_smaller(expr: &SurfaceExpr, first_param: &str) -> bool {
    let SurfaceExpr::Call { target, args, .. } = expr else {
        return false;
    };
    if target != "std::int::sub" || args.len() != 2 {
        return false;
    }
    let lhs_is_param = matches!(
        &args[0],
        SurfaceExpr::Var { name, .. } if name == first_param
    );
    let rhs_is_positive = matches!(
        &args[1],
        SurfaceExpr::IntLit { value, .. } if *value > 0
    );
    lhs_is_param && rhs_is_positive
}

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::IntLit { span, .. }
        | SurfaceExpr::BoolLit { span, .. }
        | SurfaceExpr::StringLit { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::If { span, .. } => span,
    }
}

/// Compute the set of user function names that participate in a
/// mutual recursion cycle (a call-graph SCC of size > 1). Returns
/// the set of names to reject at lowering time.
///
/// Singleton SCCs (a function that directly calls itself, or a
/// function that doesn't call any function at all) are not in the
/// returned set — direct recursion is handled by `is_recursive` +
/// `descent_provable`, and non-recursive functions are fine.
///
/// Algorithm: for each function, BFS its forward call set. If two
/// distinct functions f and g both reach each other, they (and any
/// function transitively in the same cycle) are mutually recursive.
/// Brute-force O(n²) reachability is fine for M0's small modules;
/// a proper Tarjan's SCC pass is worth doing when module sizes
/// grow or when we need to report the specific cycle.
fn compute_mutually_recursive(items: &[SurfaceItem]) -> HashSet<String> {
    let fn_names: HashSet<String> = items
        .iter()
        .filter_map(|i| match i {
            SurfaceItem::Fn { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    // Build calls[f] = set of fn names f's body calls directly.
    let mut calls: HashMap<String, HashSet<String>> = HashMap::new();
    for item in items {
        if let SurfaceItem::Fn { name, body, .. } = item {
            let mut callees = HashSet::new();
            collect_calls(body, &fn_names, &mut callees);
            calls.insert(name.clone(), callees);
        }
    }

    // For each fn f, compute reachable(f) — the transitive closure
    // of calls. Then f and g are mutually recursive iff f ∈ reachable(g)
    // AND g ∈ reachable(f) AND f != g. Direct self-recursion (f ∈
    // reachable(f) via the self-edge) is NOT in this set — it's
    // handled by `is_recursive` downstream.
    let mut reach_cache: HashMap<String, HashSet<String>> = HashMap::new();
    for f in &fn_names {
        reach_cache.insert(f.clone(), transitive_reach(f, &calls));
    }

    let mut mutually = HashSet::new();
    for f in &fn_names {
        let reach_f = &reach_cache[f];
        for g in reach_f {
            if g == f {
                continue;
            }
            let reach_g = &reach_cache[g];
            if reach_g.contains(f) {
                mutually.insert(f.clone());
                mutually.insert(g.clone());
            }
        }
    }
    mutually
}

fn transitive_reach(
    start: &str,
    calls: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = vec![start.to_string()];
    while let Some(f) = queue.pop() {
        if !visited.insert(f.clone()) {
            continue;
        }
        if let Some(callees) = calls.get(&f) {
            for c in callees {
                queue.push(c.clone());
            }
        }
    }
    // Strip the start node itself from the reach set so "f ∈
    // reachable(f)" means "f can be reached via at least one
    // non-trivial path." This matters for the singleton-direct-
    // recursion case: f ∈ calls[f] means reach(f) contains f via
    // the self-edge, which is direct recursion (not mutual).
    visited.remove(start);
    visited
}

fn collect_calls(
    expr: &SurfaceExpr,
    fn_names: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    match expr {
        SurfaceExpr::IntLit { .. }
        | SurfaceExpr::BoolLit { .. }
        | SurfaceExpr::StringLit { .. }
        | SurfaceExpr::Var { .. } => {}
        SurfaceExpr::Call { target, args, .. } => {
            if fn_names.contains(target) {
                out.insert(target.clone());
            }
            for a in args {
                collect_calls(a, fn_names, out);
            }
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_calls(cond, fn_names, out);
            collect_calls(then_branch, fn_names, out);
            collect_calls(else_branch, fn_names, out);
        }
    }
}
