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
// M1 task (1) name resolution: surface-level strings for type
// names and function targets (e.g., "Int", "std::int::add") are
// resolved to [`DeclarationId`]s via `dag.declaration_by_name()`
// at lowering time. Primitives are pre-populated by
// `bootstrap_primitives`; user functions register themselves
// with `dag.register_declaration` before their bodies are lowered
// (so recursive calls can look up the function's own signature
// without a fixpoint cycle), and patch their `body_port` with
// `dag.set_function_body_port` after the Bind is created.
//
// Surface -> L1 map:
//   IntLit             -> Value(Literal { ty: Int,    data: Int(_) })
//   BoolLit            -> Value(Literal { ty: Bool,   data: Bool(_) })
//   StringLit          -> Value(Literal { ty: String, data: String(_) })
//   Var (local)        -> scope lookup (no new node; reuses producer's port)
//   Var (unresolved)   -> placeholder port + ResolveError diagnostic
//   Call               -> Transform { target: FunctionRef, inputs }
//                         (operators like `+` pre-resolved to
//                         "std::int::add" etc. by parse.rs, then
//                         resolved to a DeclarationId here)
//   If/then/else       -> Branch with 2 Paths
//   Fn item            -> Bind with non-empty params field
//                         + Loop wrapper when body is recursive
//   Let item           -> Bind with empty params field

use std::collections::{HashMap, HashSet};

use crate::dag::{
    Behavior, BindNode, Bound, BranchNode, Dag, DeclKind, Declaration, FunctionRef, Literal,
    LiteralData, LoopNode, NodeId, Path, PortId, TransformNode, TypeShape, ValueNode,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::parse::{SurfaceExpr, SurfaceItem, SurfaceModule, SurfaceParam, SurfaceType};

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
                match lower_type(dag, ty) {
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
    let int_sentinel = lookup_primitive(dag, "Int");
    let mut param_ports: Vec<PortId> = Vec::with_capacity(params.len());
    let mut param_types: Vec<TypeShape> = Vec::with_capacity(params.len());
    let mut body_scope: HashMap<String, PortId> = outer_scope.clone();
    for param in params {
        let port = dag.alloc_port(None);
        let ty = match lower_type(dag, &param.ty) {
            Ok(ty) => {
                dag.set_port_type(port, ty);
                ty
            }
            Err(diag) => {
                dag.mark_unresolved(port, diag);
                int_sentinel
            }
        };
        body_scope.insert(param.name.clone(), port);
        param_ports.push(port);
        param_types.push(ty);
    }

    // 2. Register the function's declared signature BEFORE lowering
    //    the body, so recursive Transforms in the body can resolve
    //    their own function's return type without a cycle. We
    //    register a user-function Declaration with body_port: None
    //    and patch it in step 5 once the Bind is created.
    //
    //    On unknown return type name, we still register (with a
    //    sentinel) so that the structure of the DAG is complete,
    //    but we also allocate a placeholder port to carry the
    //    ResolveError so the fail-closed invariant holds.
    let return_ty = match lower_type(dag, return_type) {
        Ok(ty) => ty,
        Err(diag) => {
            let err_port = dag.alloc_port(None);
            dag.mark_unresolved(err_port, diag);
            int_sentinel
        }
    };
    let fn_decl_id = dag.register_declaration(Declaration {
        name: name.to_string(),
        kind: DeclKind::Function {
            params: param_types,
            return_type: return_ty,
            body_port: None,
        },
    });

    // 2.5. Mutual recursion check. If this function is part of a
    //      call cycle of size > 1, skip body lowering entirely and
    //      emit a rejection Bind whose value port is Unresolved.
    //      The declaration remains registered (so other functions
    //      that call this one can look it up and cascade Fail),
    //      but the Bind.value port's Unresolved state prevents
    //      callers from trusting the signature.
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
        dag.set_function_body_port(fn_decl_id, err_port);
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
            dag.set_port_type(loop_output, return_ty);
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

    // 5. Create the Bind for the function and patch the declaration
    //    to point at its value port. The value port's type is the
    //    declared return type — pre-set so inference for call sites
    //    can trust it (subject to the body-port check in infer's
    //    Transform decide, which catches body/signature mismatches).
    dag.set_port_type(value_port, return_ty);
    let bind_id = dag.alloc_node_id();
    dag.push_node(Behavior::Bind(BindNode {
        id: bind_id,
        name: name.to_string(),
        value: value_port,
        params: param_ports,
        span: body_span,
    }));
    dag.set_function_body_port(fn_decl_id, value_port);
    let mut outer_scope = outer_scope;
    outer_scope.insert(name.to_string(), value_port);
    outer_scope
}

/// Resolve a surface type reference to a [`TypeShape`] by looking
/// up the named declaration on the Dag. Fails with a ResolveError
/// if the name is not in the declaration table or refers to a
/// non-type declaration.
fn lower_type(dag: &Dag, ty: &SurfaceType) -> Result<TypeShape, Diagnostic> {
    match ty {
        SurfaceType::Named { name, span } => {
            let decl_id = dag
                .declaration_by_name(name)
                .ok_or_else(|| Diagnostic::ResolveError {
                    name: format!("unknown type `{name}`"),
                    span: span.clone(),
                })?;
            match dag.declaration(decl_id).kind {
                DeclKind::Type => Ok(TypeShape::new(decl_id)),
                DeclKind::Function { .. } => Err(Diagnostic::ResolveError {
                    name: format!("`{name}` is a function, not a type"),
                    span: span.clone(),
                }),
            }
        }
    }
}

/// Look up a primitive type by name. Panics if the declaration is
/// missing — bootstrap_primitives is the authority for these names
/// and a missing declaration is a bootstrap bug, not user input.
fn lookup_primitive(dag: &Dag, name: &str) -> TypeShape {
    let id = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("primitive `{name}` missing from bootstrap"));
    TypeShape::new(id)
}

fn lower_expr(
    expr: &SurfaceExpr,
    dag: &mut Dag,
    scope: &HashMap<String, PortId>,
) -> PortId {
    match expr {
        SurfaceExpr::IntLit { value, span } => {
            let ty = lookup_primitive(dag, "Int");
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                literal: Literal {
                    ty,
                    data: LiteralData::Int(*value),
                },
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::BoolLit { value, span } => {
            let ty = lookup_primitive(dag, "Bool");
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                literal: Literal {
                    ty,
                    data: LiteralData::Bool(*value),
                },
                output,
                span: span.clone(),
            }));
            output
        }
        SurfaceExpr::StringLit { value, span } => {
            let ty = lookup_primitive(dag, "String");
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Value(ValueNode {
                id: node_id,
                literal: Literal {
                    ty,
                    data: LiteralData::String(value.clone()),
                },
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
            let target_ref = match resolve_function_ref(dag, target) {
                Ok(fn_ref) => fn_ref,
                Err(()) => {
                    // Unknown target: allocate a sentinel declaration
                    // entry is NOT what we want — we want to mark the
                    // output port Unresolved so the downstream
                    // behavior survives a missing target. But we still
                    // need a valid FunctionRef for the TransformNode.
                    // Resolution: mark the output port Unresolved
                    // with a ResolveError and use a sentinel
                    // FunctionRef pointing at the first declaration
                    // (which exists because bootstrap pre-populates
                    // the table). The Transform's output is
                    // Unresolved regardless of inference, so the
                    // sentinel FunctionRef is never actually read by
                    // a downstream consumer that trusts the
                    // signature.
                    dag.mark_unresolved(
                        output,
                        Diagnostic::ResolveError {
                            name: target.clone(),
                            span: span.clone(),
                        },
                    );
                    sentinel_function_ref(dag)
                }
            };
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: target_ref,
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

/// Resolve a surface function-call target name to a [`FunctionRef`].
/// Looks up the name in the Dag's declaration table and verifies
/// that the declaration is a Function (not a Type). Returns `Err(())`
/// if the declaration is missing or the wrong kind — the caller
/// surfaces the specific diagnostic with the call-site span.
fn resolve_function_ref(dag: &Dag, name: &str) -> Result<FunctionRef, ()> {
    let id = dag.declaration_by_name(name).ok_or(())?;
    match dag.declaration(id).kind {
        DeclKind::Function { .. } => Ok(FunctionRef::new(id)),
        DeclKind::Type => Err(()),
    }
}

/// A known-valid FunctionRef used when the real target can't be
/// resolved but a TransformNode still needs a syntactically-valid
/// field. The output port is always marked Unresolved before this
/// sentinel is placed, so no consumer trusting the signature ever
/// actually reads it.
fn sentinel_function_ref(dag: &Dag) -> FunctionRef {
    // "std::int::add" is pre-populated by bootstrap_primitives and
    // is always the first function declaration registered. Using
    // it as a sentinel means the field is structurally valid even
    // when the real target was unresolvable.
    let id = dag
        .declaration_by_name("std::int::add")
        .expect("bootstrap registered std::int::add");
    FunctionRef::new(id)
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
