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

---

# Invariant Review Findings

## Previously fixed (README.md invariant tracking)

- Fixed: invariant 1 layout wording no longer points at `src/daglang/`; the root README now describes the numbered pipeline folders.
- Fixed: invariant 2 no longer names the nonexistent `src/lib/transport/`; it now points at `src/08_materialize/transport/`.
- Fixed: invariant 4 compile receipts are seeded from the already-loaded `ModuleGraph` source text instead of rereading files during receipt generation.
- Fixed: invariant 4 exec-runtime workspace path discovery moved out of `daglang-emit` and into `daglang-driver` preparation, so the emit stage is render-only.
- Fixed: invariant 6 no longer points at the nonexistent `src/gunbc-app/`; the README now describes `src/08_materialize/` as the runtime wiring layer.

---

## Open — identified 2026-03-09

### P1 — Lowerer silently drops wiring failures via `eprintln!` + `continue`

**Invariants**: #4 (phases are pure), "no hacks or fallbacks"

Three sites in `src/05_graph/daglang-lower/src/lib.rs` where a `lower_expr` error is caught, printed to stderr, and silently skipped — producing a structurally incomplete DAG instead of failing the compilation.

| Site | Function | Behavior |
|------|----------|----------|
| ~L8196 | `wire_service_call_argument` | Prints warning, returns `false` (caller ignores) |
| ~L9922 | `wire_fn_call_arguments` | Catches `Err`, prints, `continue`s — arg not wired |
| ~L11712 | `wire_callable_return_outputs` | Catches `Err`, prints, `continue`s — output not wired |

**Impact**: DAGs missing edges. Downstream execution may produce wrong results or silently skip nodes that should have run.

**Fix direction**: Accumulate errors into a diagnostics vec, propagate as `Err` from the lowering phase.

---

### P2 — Emit phase falls back on unknown types

**Invariants**: #8 (correctness by construction), "no hacks or fallbacks"

`src/07_emit/daglang-emit/src/type_mapping.rs` — `emit_identity_type` (L164–222) handles unknown type names by printing a warning and returning the name verbatim:

```rust
unknown => {
    eprintln!("warning: unknown type '{unknown}' defaulting to ValueBacking::Json");
    return unknown.to_string();
}
```

All three backends (Rust, Go, C) have this fallback. The test `identity_type_unknown_emits_name_verbatim` explicitly validates it, so it's intentional — but it contradicts Invariant 8.

**Fix direction**: Return `Result<String, EmitError>` or require the caller to register all types before emission.

---

### P3 — `Value::Skipped` → fabricated default values

**Invariant**: "no hacks or fallbacks" — explicitly names `Value::Skipped`

`src/08_materialize/resolve/src/service_ops/service_ops_impl.rs`:

- `GenericRestPrepareOp::execute` (L38–51): Returns `Value::Skipped` when inputs are skipped or required inputs missing.
- `GenericRestParseOp::execute` (L280–287): On skipped response, produces defaults via `default_output_value()`.
- `GenericShellPrepareOp::execute` (L530–548): Same skip propagation.
- `GenericShellParseOp::execute` (L714–747): On skipped response, returns `false` / empty strings / empty lists.

`default_output_value` (L500–514) produces: `Int → 0`, `Bool → false`, `String → ""`. These are valid-looking but semantically wrong.

**Status**: Partially acknowledged — `src/00_foundation/ir/src/value.rs` L183–189 documents `ControlFlow` as the replacement enum, but migration is incomplete.

**Fix direction**: Complete `ControlFlow` migration; skipped branches should produce `ControlFlow::Skipped`, not fabricated values.

---

### P4 — `writeln!().ok()` swallows I/O errors

**Invariant**: "no hacks or fallbacks" — `.ok()` on fallible operations

`src/09_execute/exec/src/ci_context.rs` L146:

```rust
writeln!(self.writer, "{}", formatted).ok();
```

Silently swallows any I/O error when emitting CI workflow commands.

**Severity**: Low — CI output is arguably best-effort, and crashing on a write failure during execution would be worse. But it does technically violate the coding standard.

**Fix direction**: If CI output is genuinely best-effort, document that exception explicitly (like the caching exception in `src/README.md`).

---

## What passed (2026-03-09 review)

- **Invariant 7**: `lower_expr` in `lib.rs` L10053–10449 — exhaustive match, no wildcard, returns `Result`. Same for `expr.rs` L268–442.
- **`extern func`**: Properly rejected at parse time (`src/03_source/daglang-syntax/src/parser.rs` L1465).
- **Phases 02–04, 06**: No `eprintln!`, no I/O side effects — clean pure functions.
- **I/O boundary** (Invariant 2): Only `08_materialize/transport/` performs direct I/O.
- **No backdoors**: Compiler provides metadata through output types, not callbacks.

---

## Scenario backlog

### P2 — `gist_recent` modeled a time cutoff as a git ref

**Date:** 2026-03-09
**Status:** Fixed in the DSL model; kept here as a preventable case.

`gunbc.tools.gist::gist_recent` accepted `since: "3.days.ago"` but called a
DSL extdep op named `git.Core.RevListBase`. In the DSL model, that op was
wired to:

```dag
transport shell { argv: ["git", "merge-base", "HEAD", "{since}"] }
```

At runtime, `3.days.ago` was treated as an object name instead of a time
expression, producing:

```text
fatal: Not a valid object name 3.days.ago
```

What made this preventable:

- The lower Rust git transport already had the correct concept,
  `RevListBefore(before)` using `git rev-list -1 --before=... HEAD`, with
  tests.
- The DSL extdep model drifted from that transport model and nothing enforced
  parity.
- A hermetic temp-repo execution test for `gunbc.tools.gist::gist_recent`
  would have exercised the real git command before manual use.

Inspiration only:

- Add parity checks between DSL extdep service definitions and lower transport
  models.
- Generate hermetic git-backed integration tests for DAG tools that consume
  date/ref inputs.
- Strengthen git-facing types so time expressions and refs cannot flow through
  the same `String` slot unnoticed.

### P1 — bootstrap swallowed a callable-arg lowering failure and crashed later

**Date:** 2026-03-09
**Status:** Fixed in lowering; kept here as a preventable case.

`tools.bootstrap` called `render_gitignore_file(file: gitignore_file, ...)`,
where `gitignore_file` was a local record binding with a pipe-based field
expression. The lowerer hit:

```text
warning: cannot wire fn call argument 'file' in tools.bootstrap.bootstrap: ...
```

and continued lowering. That produced a DAG where the pure fn callable
executed without its `file` input wired. Runtime then failed later in
bootstrap with:

```text
FnBody evaluation failed with real inputs present: eval error: unbound variable: file
```

What made this preventable:

- The compiler already knew the `file` arg wiring had failed.
- The warning came from a swallowed lowering error, not from an unknown runtime
  condition.
- The expression itself was still evaluable by the fn-body evaluator; only the
  structural lowering path had a gap.

Inspiration only:

- Fail closed when callable arg wiring fails instead of logging and continuing.
- Fall back to a helper fn-body expression node for complex pure arg
  expressions when structural lowering is incomplete.
