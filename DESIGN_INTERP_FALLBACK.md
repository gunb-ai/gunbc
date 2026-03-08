# Design: Eliminating Interpreter Fallback from the Compiler

**Status:** Problem catalog. No solution adopted yet.

**Invariant:** README Invariant 7 — every expression lowers to structural DAG
nodes or the compilation fails. No interpreter-backed fallback nodes.

**Scope:** The lowerer currently has 18 sites that produce interpreter-backed
nodes (`ExprCompute`, `PipeOp`, `ForOp`) instead of failing when structural
lowering cannot succeed. This document catalogs each site, identifies the
underlying problem, and groups them by root cause so a coherent solution can
be designed.

**Relationship to POSTMORTEM.md:** T9 (local variable wiring), T13
(zero-fallback), and the T9 TBD note ("narrow or remove interpreter-backed
fallback nodes") are the origin of this work. This document supersedes the
TBD note with a concrete catalog.

---

## Background

The DAG language compiles `.dag` source to a graph IR (`Dag<LoweredOp>`)
where every node has typed ports and every data dependency is an edge. The
lowerer is responsible for this transformation. When it succeeds, the graph
is fully structural: the emitter can compile it, the executor can run it,
and the graph shape encodes the full computation.

When the lowerer cannot structuralize an expression, it currently calls
`synthesize_expr_compute()` (or `synthesize_tagged_evaluator()` for pipe/for),
which emits a node carrying the raw expression AST. This node is evaluated at
runtime by `evaluate_fn_body()` in the resolve layer — a mini-interpreter
that walks the AST with a value environment.

This creates three problems:

1. **The emitter cannot compile these nodes.** It emits `Passthrough` stubs
   (`Ok(inputs)`) that forward inputs unchanged — semantically wrong.

2. **The resolver masks failures.** `ExprComputeOp::execute()` catches
   "unknown function" errors and returns `Value::Skipped`. `FnBodyCallableOp`
   catches all errors and returns `Skipped` for all outputs. Failures become
   invisible.

3. **The graph shape is a lie.** An `ExprCompute` node with an opaque
   `fn_body` hides the real data dependencies inside an AST blob. Upstream
   analyses (reachability, purity, structural diffing) cannot see through it.

---

## Catalog of Interpreter Fallback Sites

### Notation

Each site is in `src/05_graph/daglang-lower/src/lib.rs` unless noted. Line
numbers are approximate. Sites are grouped by root cause.

---

### Root Cause A: Unresolvable local let bindings (`has_local_refs`)

When `collect_expr_leaf_refs()` encounters an identifier that is not a param,
callable, service, endpoint, or known local let binding, it sets
`has_local_refs = true`. Several synthesis functions check this flag and
fall back to `ExprCompute` immediately, before attempting structural lowering.

The fundamental issue: the lowerer tracks five source maps (`param_types`,
`bound_callable_sources`, `bound_service_sources`, `expanded_results`,
`endpoints_by_name`) and one local binding map (`local_let_bindings`). An
identifier that isn't in any of these cannot be wired as a DAG edge.
Identifiers that fall through include: loop variables, match arm bindings,
lambda parameters, and variables from inner scopes that aren't in
`local_let_bindings` (which only tracks top-level `let` statements in the
current body).

| # | Site | Line | Trigger |
|---|------|------|---------|
| A1 | `synthesize_match_dispatch` | ~11109 | `has_local_refs` in any match arm body |
| A2 | `synthesize_conditional` | ~11278 | `has_local_refs` in condition, then, or else |

**DSL example triggering A1/A2:**

```
fn format_result(result: Result) -> String {
    let label = match result {
        Ok { value } => "success: {value}"    // `value` is arm binding — not in source maps
        Err { message } => "error: {message}"
    }
    return { formatted: label }
}
```

The match arm bindings `value` and `message` are not in any source map. The
lowerer sees `has_local_refs = true` and falls back to `ExprCompute` for the
entire match expression.

**Why this is hard:** Match arm bindings and lambda parameters are genuinely
local to a sub-expression. They don't correspond to DAG nodes — they're bound
by the pattern/lambda and only meaningful inside the arm/body. A structural
`MatchDispatch` node handles this correctly when the arm bodies only reference
the binding and known DAG sources. But when an arm body contains nested
expressions that reference the binding (e.g., string interpolation), the
current leaf-ref collector can't distinguish "this is an arm binding that
the MatchDispatch will provide" from "this is a truly unresolvable reference."

---

### Root Cause B: Sub-expression resolution failure in structural synthesis

Each structural synthesis function (`synthesize_conditional`,
`synthesize_match_dispatch`, `synthesize_binary_op`, etc.) tries to resolve
its sub-expressions to `(node_id, port)` pairs via `resolve_return_expr_source`.
If any sub-expression returns `None`, the whole expression falls back to
`ExprCompute`.

The underlying issue is that `resolve_return_expr_source` returns `Option` —
it either finds a DAG source or it doesn't. There's no partial success, no
error, and no indication of *why* resolution failed. The caller can only
give up and fall back.

| # | Site | Line | Trigger |
|---|------|------|---------|
| B1 | `synthesize_match_dispatch` (scrutinee) | ~11186 | Scrutinee expr can't be resolved |
| B2 | `synthesize_conditional` (condition) | ~11316 | Condition expr can't be resolved |
| B3 | `synthesize_conditional` (then branch) | ~11336 | Then expr can't be resolved |
| B4 | `synthesize_conditional` (else branch) | ~11356 | Else expr can't be resolved |
| B5 | `resolve_return_expr_source` (BinOp) | ~10397 | Left or right operand can't be resolved |
| B6 | `resolve_return_expr_source` (UnaryOp) | ~10450 | Inner operand can't be resolved |
| B7 | `synthesize_variant_construct` | ~11499 | Any variant field can't be resolved |
| B8 | `synthesize_record_construct` | ~11586 | Any record field can't be resolved |
| B9 | `synthesize_list_construct` | ~11665 | Any list element can't be resolved |
| B10 | `synthesize_string_interpolate` | ~11758 | Any interpolated expr can't be resolved |

Resolution failure typically means the sub-expression is itself a local let
binding (Root Cause C) or a complex expression that recursively hits one of
the other root causes.

---

### Root Cause C: Local let binding as return source

When `resolve_return_expr_source` encounters an `Expr::Ident(name)` where
`name` is a local let binding, it calls `synthesize_expr_compute` directly on
the bound expression. It does not attempt to resolve the bound expression
structurally first.

| # | Site | Line | Trigger |
|---|------|------|---------|
| C1 | `resolve_return_expr_source` (Ident) | ~10204 | Name found in `local_let_bindings` |
| C2 | `wire_fn_call_arguments` | ~10022 | Arg is a local let binding passed to fn call |

**DSL example triggering C1:**

```
fn build_stages(input: Input) -> StageList {
    let stages = [stage_from_output(input.first), stage_from_output(input.second)]
    return { stages: stages }
}
```

`stages` is a local let binding. When the return expression references it,
`resolve_return_expr_source` finds it in `local_let_bindings` and immediately
calls `synthesize_expr_compute` on the bound expression `[stage_from_output(...), ...]`.

**Why this is wrong:** The bound expression `[stage_from_output(...), ...]` is a
list construction with callable sub-expressions — it has a perfectly valid
structural lowering via `synthesize_list_construct` + recursive resolution.
But `resolve_return_expr_source` doesn't try that path for local let bindings.
It goes straight to `ExprCompute`.

**The fix direction:** When encountering a local let binding, recursively call
`resolve_return_expr_source` on the bound expression instead of
`synthesize_expr_compute`. This is essentially what the `Expr::FieldAccess`
branch already does at line ~10232 (it resolves through the binding and then
applies `GetField` structurally). The `Expr::Ident` branch should do the same.

---

### Root Cause D: Service call argument wiring

When `wire_service_call_arg_to_port` cannot wire an argument by identifier
lookup, field access, direct call, or literal, it falls back to
`synthesize_expr_compute` on the raw argument expression.

| # | Site | Line | Trigger |
|---|------|------|---------|
| D1 | `wire_service_call_arg_to_port` | ~8291 | Complex expression as service call arg |

**DSL example triggering D1:**

```
service.Create(
    name: "{prefix}_{suffix}",     // string interpolation — complex expr
    count: a + b,                   // binary op — complex expr
)
```

The argument wiring code checks for simple cases (ident, field access, call,
literal) but doesn't attempt structural synthesis for compound expressions.
It goes straight to `ExprCompute`.

**The fix direction:** Call `resolve_return_expr_source` (which attempts
structural synthesis) instead of `synthesize_expr_compute` directly.

---

### Root Cause E: No structural lowering attempted

Some expression forms go straight to the interpreter without attempting
structural lowering. This includes both explicitly-tagged forms (Pipe, For)
and — critically — a `_` catch-all that silently swallows every `Expr`
variant that isn't explicitly matched.

**The catch-all is a structural invariant violation.** The `Expr` enum has
18 variants. `resolve_return_expr_source` explicitly handles 10 of them.
The remaining 8 fall through to `_ => synthesize_expr_compute(...)`:

| Expr variant | What it is | Should lower to |
|---|---|---|
| `Pipe` | `expr \|> fn` | Desugar to `Call` and resolve structurally |
| `PipeCall` | `expr \|> method(args)` | Desugar to `Call` and resolve structurally |
| `For` | `for x in list { body }` | Structural map/collection node |
| `Call` | `f(a, b)` | Callable endpoint wiring (already done elsewhere in lowerer) |
| `ServiceCall` | `svc.Op(args)` | Service endpoint wiring (already done elsewhere in lowerer) |
| `Lambda` | `x => body` | Anonymous callable / inline sub-expression |
| `Map` | `{ "key": val }` | Structural map construction |
| `Guarded` | `expr [when cond]` | Conditional wrapper |
| `After` | `expr [after dep]` | Dependency edge (not a value-producing node) |
| `Return` | `return { fields }` | Record construction + output wiring |

Pipe and For are explicitly tagged (`PipeOp`, `ForOp`) for tracking purposes,
but all three categories (Pipe/For, the 7 catch-all variants, and the
catch-all itself) share the same fundamental problem.

| # | Site | Line | Trigger |
|---|------|------|---------|
| E1 | `resolve_return_expr_source` (Pipe) | ~10481 | `expr \|> fn()` pipe expression |
| E2 | `resolve_return_expr_source` (PipeCall) | ~10481 | `expr \|> fn(arg: val)` pipe call |
| E3 | `resolve_return_expr_source` (For) | ~10491 | `for x in list { body }` expression |
| E4 | `resolve_return_expr_source` (catch-all) | ~10501 | `Call`, `ServiceCall`, `Lambda`, `Map`, `Guarded`, `After`, `Return` |

**This violates multiple README invariants beyond Invariant 7:**

- **Invariant 2 (I/O is structural):** A `ServiceCall` inside an expression
  that hits the catch-all gets bundled into an opaque `ExprCompute` blob.
  The I/O is no longer visible in the graph structure — you can't tell by
  looking at the DAG that this node does I/O. Dry-run can't intercept it.
  Transport mocking can't reach it.

- **Invariant 4 (each phase is a pure function):** The lowering phase is
  supposed to transform every valid AST construct into DAG IR. For 8 of 18
  expression forms, it defers the work to the resolve layer at runtime.
  The lowerer isn't completing its job.

- **Invariant 8 (construction, not validation):** The `_` catch-all is a
  wildcard that silently accepts any new `Expr` variant added to the parser.
  If someone adds `Expr::TypeCast` tomorrow, it will silently land in
  `ExprCompute` with no compiler error, no test failure, no indication that
  structural lowering was skipped. The match should be exhaustive with no
  wildcard — every variant handled explicitly or rejected at compile time.

**This is not common practice in the compiler.** The 10 variants that *are*
handled all have dedicated structural synthesis functions with proper
`PrimitiveOpKind` nodes. The C24 comments throughout the code show this was
being built incrementally. Pipe and For are explicitly acknowledged as
temporary (`"tagged distinctly so we can track ExprCompute elimination
progress"`). But the `_` catch-all hiding 7 additional unhandled variants
is a larger gap than the tagged forms — it's the silent kind of problem
that Invariant 8 exists to prevent.

---

### Root Cause F: `synthesize_tagged_evaluator` unreachable fallback

The `synthesize_tagged_evaluator` function has a catch-all branch that produces
`ExprCompute` when `kind_tag` is neither `"pipe"` nor `"for"`.

| # | Site | Line | Trigger |
|---|------|------|---------|
| F1 | `synthesize_tagged_evaluator` (catch-all) | ~12021 | `kind_tag` is not "pipe" or "for" |

This branch is currently unreachable — the function is only called with
`"pipe"` or `"for"` from `resolve_return_expr_source`. It exists as defensive
code. Under Invariant 7 this should be an `unreachable!()` or removed entirely.

---

## Summary by Root Cause

| Cause | Sites | Nature | Fix complexity |
|-------|-------|--------|----------------|
| **A: `has_local_refs`** | 2 | Arm/lambda bindings treated as unresolvable | Medium — leaf-ref collector needs scope-aware binding tracking |
| **B: Sub-expression resolution failure** | 10 | Cascading `None` from recursive resolution | Low — these are symptoms; fix the causes (C, D, E) and B sites resolve |
| **C: Local let binding as return source** | 2 | Direct `ExprCompute` instead of recursive structural resolution | Low — change `Ident` branch to recurse like `FieldAccess` does |
| **D: Service call arg wiring** | 1 | Bypasses structural synthesis entirely | Low — call `resolve_return_expr_source` instead of `synthesize_expr_compute` |
| **E: No structural lowering attempted** | 4 (covering 10 `Expr` variants) | Pipe, For go straight to interpreter; `_` catch-all silently swallows `Call`, `ServiceCall`, `Lambda`, `Map`, `Guarded`, `After`, `Return` | Medium–High — requires desugaring, new primitives, and exhaustive match |
| **F: Unreachable fallback** | 1 | Dead code | Trivial — delete or `unreachable!()` |

Total: 20 sites (18 reachable).

---

## Dependency Order

Root causes are not independent. Fixing one unlocks others:

```
C (local let → recurse structurally)
  └─ unlocks B5–B10 (sub-expression resolution succeeds for let-bound exprs)
       └─ unlocks A1–A2 (fewer `has_local_refs` triggers)

D (service arg → structural synthesis)
  └─ independent, fixes D1 directly

E (pipe/for/catch-all → desugar or new primitives)
  └─ independent, fixes E1–E4 directly
       └─ also resolves some B sites when pipe/for appears as sub-expression

F (unreachable fallback)
  └─ trivial cleanup, no dependencies
```

The suggested order:

1. **C first.** Highest leverage: two sites, low complexity, unlocks cascade.
   Change `resolve_return_expr_source`'s `Expr::Ident` branch to recurse
   through local let bindings structurally (like `FieldAccess` already does).
   Change `wire_fn_call_arguments` similarly.

2. **D next.** One site, low complexity, independent.

3. **A after C.** Once C is fixed, many `has_local_refs` cases disappear
   because the let binding itself resolves structurally and its transitive
   references become visible. The remaining A cases (arm bindings, lambda
   params) need the leaf-ref collector to understand scoped bindings.

4. **E last.** Requires new structural primitives or desugaring. Pipe is
   likely a desugar to `Call`. For requires either a structural `Map` node
   or desugar to a collection operation. The catch-all needs each remaining
   `Expr` variant to be handled explicitly.

5. **F trivially.** Delete the unreachable branch.

After all causes are resolved, `synthesize_expr_compute` and
`synthesize_tagged_evaluator` have no callers and can be deleted, along with
`PrimitiveOpKind::ExprCompute`, `PrimitiveOpKind::PipeOp`, and
`PrimitiveOpKind::ForOp`. The `ExprComputeOp` in the resolver and the
`Passthrough` stubs in the emitter become dead code.

---

## Open Questions

1. **Scope of structural expressibility.** Should every DSL expression be
   structuralizable, or should some expression forms be restricted/disallowed
   in fn/func bodies? For example, `for x in list { complex_body }` might
   be better expressed as a pipeline stage than as an inline expression.

2. **Pipe desugaring.** `x |> map(fn)` desugars to `map(fn, x)`. But `map`
   is a stdlib function resolved by the evaluator, not a DAG node. Structural
   pipe requires either: (a) a structural `Map` primitive that the lowerer
   emits directly, or (b) all pipe-target functions to be resolvable as DAG
   callables.

3. **Match arm bindings as structural inputs.** A `MatchDispatch` node needs
   to provide arm bindings to arm bodies. Currently the arm body is a lowered
   expression evaluated by the dispatch op. If arm bodies themselves need
   structural lowering (e.g., an arm body that calls a service), the
   `MatchDispatch` op needs to be a sub-DAG, not a flat node. This is a
   significant structural change.

4. **Error quality.** When the lowerer rejects an expression (Invariant 7),
   the error must explain *why* and suggest alternatives. "Cannot lower
   `for x in list { body }` to structural DAG nodes" is not actionable.
   "Use a `stage` with `map` collection instead of inline `for`" is.

5. **The `_` catch-all must go.** The match in `resolve_return_expr_source`
   must be exhaustive with no wildcard. Every `Expr` variant gets an explicit
   arm — either structural synthesis or a clear compile error. This is the
   minimum structural guarantee (Invariant 8): a new `Expr` variant added to
   the parser must cause a compile-time error in the lowerer until a
   structural lowering is implemented. Today it silently falls through to
   `ExprCompute`.

6. **`Call` and `ServiceCall` in expression position.** These are already
   handled elsewhere in the lowerer (service call wiring, callable endpoint
   resolution). Their presence in the catch-all means `resolve_return_expr_source`
   doesn't recognize them — likely because they appear as nested sub-expressions
   (e.g., inside a record field or as a pipe argument) rather than as
   top-level statements. The fix may be to wire through to the existing
   lowering paths, not to create new structural primitives.
