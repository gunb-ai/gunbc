# Postmortem: Unconditional SetIamPolicy in preflight

**Date:** 2026-03-09
**Severity:** P1 (removed the code; analysis preserved here)
**Resolution:** Deleted `check_iam_binding`, `add_iam_binding`, and
`iam_preflight_check` from `dsl/std/patterns.dag`.

---

## What happened

`iam_preflight_check` was a `func` in `std/patterns.dag` that called
`GetIamPolicy`, checked whether a binding existed, and conditionally called
`SetIamPolicy` to add it. The conditional was expressed with `[when]`:

```dag
has_binding = check_iam_binding(bindings: policy.bindings, ...)

updated = add_iam_binding(bindings: policy.bindings, ...) [when !has_binding.has_role]

set_result = gcp.ResourceManager.SetIamPolicy(...) [when !has_binding.has_role]
```

Two problems combined to make `SetIamPolicy` execute unconditionally:

1. `check_iam_binding` was a stub that always returned `{ has_role: false }`,
   so the `[when !has_binding.has_role]` condition was always true.

2. The `[when]` guard on the `SetIamPolicy` service call was never lowered
   into the DAG IR, so even if `check_iam_binding` returned the correct
   value, the conditional would have been ignored at the graph level.

The result: every call to `iam_preflight_check` performed an IAM write,
which fails for identities that can read policy but not update it, causing
false-negative preflight failures.

---

## Why testgen didn't catch it

Testgen generates `GuardBranchCoverage` tests for nodes that have `Guard`
annotations in the DAG IR. Since the `[when]` guard was never lowered into
the DAG, the `SetIamPolicy` transport execute node had no guard, and testgen
had no way to know the node was supposed to be conditional.

---

## Root cause: `[when]` on `func` body service calls is not lowered

### How `[when]` works in `fn` items (correctly)

For `fn` items, the `fn` body lowering path (`expr.rs`) converts
`NodeStmt.when_guard` into an `IfElse` expression:

```rust
// expr.rs line 249
if let Some(guard) = &ns.when_guard {
    expr = LoweredExpr::IfElse {
        cond: lower_expr(guard),
        then_: expr,
        else_: LoweredLiteral::None,
    };
}
```

The function evaluator executes this at runtime: if the condition is false,
the binding gets `None`. This is pure control flow inside the function body.

### How `[when]` works in `pattern` items (via expansion)

For patterns, the body is expanded inline by the pattern expansion compiler.
`[when]` and `[after]` guards become dependency edges and If-pattern SubDags.
The pattern expansion code handles this through `expand_pattern_body_node`.

### How `[when]` works in `func` body service calls (it doesn't)

For `func` items, the body is not compiled as a function body or expanded as
a pattern. Instead, the lowerer:

1. Collects service calls from the body via `collect_service_calls_from_stmts`
2. Wires each service call to its transport triplet (prepare/execute/parse)

Step 1 uses `walk_stmts` which visits all expressions recursively but strips
context. The `[when]` guard ends up as `Expr::Guarded(ServiceCall, condition)`
inside a `Stmt::Assign`. The `ServiceCallSite` struct only captures `path`
and `args` — it has no field for the guard condition. The guard is silently
dropped.

Step 2 wires arguments from the service call to the transport prepare node.
Since no guard information survived step 1, the transport execute node gets
no conditional annotation.

### Parser representation detail

The DSL syntax `name = expr [when cond]` is parsed as
`Stmt::Assign(name, Expr::Guarded(expr, cond))`. The `[when]` is part of
the expression, not the statement. This is different from `node` statements
where the syntax is `node name [when cond]: expr`, which produces
`Stmt::Node(NodeStmt { when_guard: Some(cond), expr })`.

The `iam_preflight_check` code uses assignment syntax (no `node` keyword),
so the guard lives in `Expr::Guarded`, not in `NodeStmt.when_guard`.

---

## Why `Guard` in the DAG IR is the wrong mechanism for `[when]`

An initial fix attempt tried to propagate `[when]` as a `Guard` annotation
on the transport execute node's input port. This would have violated the
DAG IR invariant for `Guard`.

### What `Guard` is for today

`Guard` has exactly two uses in the lowered DAG:

1. **Skip-chain propagation.** Every transport execute node has
   `Guard::NotEq(Value::Skipped)` on its `request` port. When the upstream
   prepare node is skipped (because its upstream was skipped), the execute
   node also skips. This is a structural invariant of the transport triplet.

2. **Branch/If pattern routing.** `BranchBuilder` and `IfBuilder` create
   SubDag nodes with `Guard::Eq(Value::Bool(true/false))` on the condition
   port to route values through the correct arm.

Both uses are **structural** — set at compile time by the lowerer. Neither
implements user-defined conditional logic.

### Why the semantics don't match

When the executor skips a node via `Guard`, **all outputs become
`Value::Skipped`**:

```rust
// execute/mod.rs line 747
let (outputs, was_intercepted) = if skip {
    let outputs = node.outputs.iter()
        .map(|p| (p.name.0.clone(), Value::Skipped))
        .collect();
    ...
```

`Value::Skipped` is a poison value that propagates through skip chains.
Any downstream node with `Guard::NotEq(Value::Skipped)` also skips.

But `[when]` in the DSL means: if the condition is false, the binding is
`None` — a normal value that downstream expressions can branch on. The
return expression `if has_binding.has_role { true } else { set_result.ok }`
expects `set_result` to be either a real value or `None`, not `Skipped`.

Using `Guard` for `[when]` would produce `Skipped` instead of `None`,
causing cascading skips through the transport chain.

---

## Correct approach (not yet implemented)

The `[when]` on `func` body service calls should be lowered as an
**If-pattern SubDag**, the same mechanism the lowerer already uses for
`if/else` and `match` control flow in func bodies
(`add_control_flow_pattern_nodes`).

This would:

1. Detect `Expr::Guarded(ServiceCall, condition)` in `Stmt::Assign`
2. Create an If-pattern SubDag that wraps the transport triplet
3. Route through the true branch (execute the service call) or false branch
   (produce `None`)
4. Produce `None` (not `Skipped`) when the condition is false

### Prerequisites

- FC-CF5 (JSON array iteration) for a real `check_iam_binding` implementation
- Extension of `add_control_flow_pattern_nodes` to detect `Expr::Guarded`
  wrapping service calls in `Stmt::Assign` / `Stmt::Let`
- Testgen should detect constant-return `fn` stubs that feed conditional
  expressions, so the pattern is caught even if someone re-introduces it

---

## Hypothetical use case

IAM preflight is a common pattern in cloud automation: before executing a
workflow that requires specific IAM permissions, check whether the identity
already has the required role binding. If not, add it. This avoids failures
deep in a workflow due to missing permissions.

In the DAG DSL, this would look like:

```dag
func iam_preflight_check(access_token, project_id, sa, role) -> { ok: Bool } {
  policy = gcp.ResourceManager.GetIamPolicy(access_token, project_id)
  has_binding = check_iam_binding(policy.bindings, sa, role)

  // Only write if the binding is missing
  updated = add_iam_binding(policy.bindings, sa, role) [when !has_binding.has_role]
  set_result = gcp.ResourceManager.SetIamPolicy(
    access_token, project_id, updated.updated_bindings, policy.etag
  ) [when !has_binding.has_role]

  return { ok: if has_binding.has_role { true } else { set_result.ok } }
}
```

The `[when]` is essential: without it, `SetIamPolicy` fires on every call,
even when the role is already present. This turns a read-mostly preflight
into a mandatory write operation, which:

- Fails for identities with `getIamPolicy` but not `setIamPolicy` permission
- Creates unnecessary IAM policy churn
- Risks hitting IAM rate limits on high-frequency preflight checks

The `[when]` guard makes `SetIamPolicy` conditional on the check result:
execute only when the binding is missing, skip (producing `None`) when it's
already present. This is the standard check-then-act pattern that `ensure`
and `upsert` patterns in `std/patterns.dag` are designed around.

---

## What `[when]` means in the DSL

`[when condition]` on a statement means: "evaluate the expression only if
the condition is true; otherwise, bind `None` to the result."

It is syntactic sugar for:

```dag
result = if condition { some_expression } else { None }
```

It is **not** the same as DAG-level node skipping (`Guard`/`Value::Skipped`),
which is an executor mechanism for propagating transport failures through
dependency chains.

### Current support matrix

| Item type | `[when]` mechanism | Status |
|-----------|-------------------|--------|
| `fn` body | `IfElse` in fn evaluator | Works |
| `pattern` body | Pattern expansion + If SubDag | Works |
| `func` body (local calls) | Should use If SubDag | Not implemented |
| `func` body (service calls) | Should use If SubDag | Not implemented |

The gap is specifically in `func` body lowering, where `Expr::Guarded`
wrapping service calls (and local fn calls) is silently stripped by the
lowerer without creating the corresponding If-pattern SubDag.
