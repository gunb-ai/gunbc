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

## Architectural Root Cause

The individual fallback sites (cataloged below) are symptoms. The root cause
is an architectural mismatch: **the lowerer has a flat, statement-level design
but the AST is tree-structured.**

### The lowerer's three-phase architecture

The lowerer processes fn/func/pattern bodies in three phases:

**Phase 1 — Statement-level node creation.** Walks top-level statements.
For each `let name = expr` or `name = expr`:

- If `expr` is `Call(name, args)` → creates a callable endpoint DAG node,
  records `name → endpoint` in `bound_callable_sources`
- If `expr` is `ServiceCall(path, args)` → creates a transport triplet
  (prepare/execute/parse), records `name → triplet` in
  `bound_service_sources`
- **Everything else → `_ => {}` — silently skipped** (line ~4226, ~12342)

**Phase 2 — Argument wiring.** `wire_fn_call_arguments` and
`wire_service_call_arg_to_port` wire inputs to the DAG nodes created in
Phase 1. These use the same source maps (params, callables, services).

**Phase 3 — Return wiring.** `wire_callable_return_outputs` extracts
return expressions and calls `resolve_return_expr_source` to map each one
to a `(node_id, port)` pair — an existing DAG source.

### Where the design breaks

Phase 1 only creates DAG nodes for `Call` and `ServiceCall` at the
**top level** of a statement. It does not recurse into expressions. A
`ServiceCall` nested inside a record literal, a `Call` inside a conditional
branch, a pipe chain in a return expression — none of these are seen by
Phase 1. No DAG nodes are created for them.

Phase 3 then tries to resolve the return expression. It finds identifiers
that reference things Phase 1 created (`bound_callable_sources`,
`bound_service_sources`). But when the return expression contains inline
computation — calls, conditionals, pipes, string interpolation — Phase 3
can't find DAG nodes for those sub-expressions because **they were never
lowered.**

`resolve_return_expr_source` grew incrementally to fill this gap. It
started as a simple wiring function (resolve an ident to a DAG source)
and was progressively extended with structural synthesis functions
(`synthesize_conditional`, `synthesize_match_dispatch`,
`synthesize_binary_op`, etc.) to handle expressions that Phase 1 skipped.
But it was never designed to be a full lowering pass. The `_` catch-all
exists because some expression forms still have no structural synthesis,
and the function lacks the infrastructure (endpoint registries, transport
wiring, argument binding) that Phase 1 has.

### The wildcards

There are **two** wildcards, and they compound:

1. **Phase 1's `_ => {}`** (line ~4226, ~12342): If a statement's
   expression is not `Call` or `ServiceCall`, Phase 1 silently does nothing.
   The expression is not lowered. It's recorded in `local_let_bindings` by
   `collect_local_let_bindings` (the cleanup crew for Phase 1's gaps).

2. **Phase 3's `_ => synthesize_expr_compute(...)`** (line ~10501):
   If `resolve_return_expr_source` encounters an `Expr` variant it can't
   structuralize, it punts to the interpreter. This is the cleanup crew
   for Phase 3's gaps.

The first wildcard creates the problem. The second wildcard hides it.
`ExprCompute` is the mechanism by which the lowerer avoids confronting the
fact that Phase 1 didn't lower the expression.

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

## Dependency Order (Site-Level)

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

But all of these are downstream of the architectural root cause. Fixing
individual sites within `resolve_return_expr_source` treats the symptom:
the function keeps growing as a second lowering pass grafted onto a wiring
function. The site-level fixes are valid but don't resolve the structural
problem.

## Design Direction

The architectural root cause is the flat Phase 1 / Phase 3 split: Phase 1
lowers statements, Phase 3 tries to wire returns, and the gap between them
is filled by `ExprCompute`. Two possible design directions:

### Option A: Flatten before lowering (normalize the AST)

Add a pre-lowering desugaring pass that extracts nested computation from
expressions into top-level let bindings. After desugaring, every statement
is either a `Call`, a `ServiceCall`, or a simple binding to a resolvable
expression. Return expressions only reference identifiers, field accesses,
and literals — never inline computation.

Example before desugaring:
```
return { result: if cond { svc.Op(x: input) } else { fallback(y: input) } }
```

After desugaring:
```
let __branch_result = svc.Op(x: input) [when cond]
let __else_result = fallback(y: input) [when !cond]
let __merged = __branch_result ?? __else_result
return { result: __merged }
```

Phase 1 would then see `svc.Op` and `fallback` as top-level statements and
lower them normally. Phase 3 would only wire identifiers.

**Pros:** Minimal changes to the lowerer's Phase 1 architecture. The existing
statement-level lowering and wiring infrastructure handles everything.
The desugaring pass is a self-contained pre-processing step.

**Cons:** The desugared AST may not map well to all expression forms
(match arms with bindings, for loops with closures, pipe chains). Some
expressions may not have obvious desugaring targets. Error messages would
reference synthesized names (`__branch_result`) unless carefully mapped back.

### Option B: Make lowering tree-recursive

Replace the Phase 1 / Phase 3 split with a single recursive lowering pass
that walks the expression tree. Every `Expr` node produces either a
reference to an existing DAG node or a newly-created structural DAG node.
`Call` and `ServiceCall` create endpoint/transport nodes at the point
where they appear in the tree, not only when they're top-level statements.

This is what `resolve_return_expr_source` was evolving toward — but it
lacked the infrastructure (endpoint registries, transport wiring, service
resolution) that Phase 1 has. Option B would give it that infrastructure
by design rather than by incremental accretion.

**Pros:** Direct. No intermediate desugaring. Every expression form gets
a structural lowering at the point where it appears. The match in the
recursive lowering function would be exhaustive — no wildcard. New `Expr`
variants cause a compile error until handled.

**Cons:** Significant refactor. Phase 1's infrastructure (endpoint
resolution, transport triplet creation, argument wiring, resource
acquisition) would need to be available during tree-recursive lowering.
The current separation between "create nodes" and "wire edges" would
change: node creation and edge wiring would be interleaved as the tree
is walked.

### Option C: Restrict the language

Disallow nested computation in return expressions and other non-statement
positions. The DSL grammar or typechecker would reject:
- `ServiceCall` outside of a top-level statement
- `Call` outside of a top-level statement (except as arg to another call)
- `Pipe`/`For` outside of a top-level statement

This makes the Phase 1 assumption true by construction: all effectful
computation is a top-level statement, and return expressions are pure wiring.

**Pros:** Simplest. No lowerer changes. The constraint is arguably good
language design — it forces DSL authors to name intermediate results,
making the DAG shape explicit in the source.

**Cons:** Breaks existing `.dag` files that use inline computation. May
feel restrictive to DSL authors. Pipe chains (`x |> map(fn) |> filter(fn)`)
are a core ergonomic feature and restricting them to statement position
removes much of their value.

### Key finding: Collection lowering already exists

The infrastructure to lower pipe chains to structural DAG nodes **already
exists** in Phase 1:

- `CollectionKind` enum (Map, Filter, Fold, Join, FlatMap, Sort, etc.)
- `LoweredOp::Collection { module, callable, kind }` — the structural node
- `collect_collection_ops_from_stmts` — walks statements, finds pipes
- `build_collection_lowering_plan` — emits chained Collection nodes
- The emitter compiles these as `Computation::Collection`
- Tests verify `stages |> map() |> filter() |> join()` → three chained nodes

This is exactly the design intent: pipe syntax is shorthand for graph IR.
**But this infrastructure is only used in Phase 1.** When the same pipe
expression appears in a return field, record literal, or nested inside a
conditional, Phase 3 (`resolve_return_expr_source`) hits `Expr::Pipe` →
`synthesize_tagged_evaluator` → `PipeOp` → interpreter. It bypasses
the Collection lowering entirely.

The problem is not "how do we lower pipes structurally." That's solved.
The problem is "why do pipes reach Phase 3 at all when Phase 1 already
knows how to handle them."

### Recommendation

**Option C (restrict the language) is the natural fit.** The DSL's design
intent is that everything is a DAG. Pipe chains already lower to structural
Collection nodes when they're top-level statements. The compiler should
require them to be top-level. Inline pipe chains (in return fields, record
literals, conditional branches) should be disallowed — the DSL author must
name the intermediate result with a `let` binding, which makes it a
statement that Phase 1 can lower.

This aligns with the existing architecture rather than fighting it. It
requires no new lowering infrastructure. It makes the DAG shape explicit
in the source. And it eliminates the need for `PipeOp`, `ForOp`, and
`ExprCompute` because the expressions that trigger them would no longer
be valid DSL.

The remaining pure expressions (string interpolation, arithmetic,
comparisons, record/list construction) already have structural
`PrimitiveOpKind` equivalents. Their fallback to `ExprCompute` only
triggers when a sub-expression can't be resolved — which is typically
because a pipe chain or lambda is nested inside them. Once pipes are
restricted to statement position, these structural synthesis functions
should succeed without fallback.

~~Option A (flatten) achieves the same result automatically but hides the
transformation from the DSL author. Option B (tree-recursive) is the
most general but the largest change. Option C is the simplest and most
aligned with the language's design philosophy.~~

**Update:** Option C (restrict the language) is wrong. `return { total:
stages |> count() }` is trivially equivalent to `let total = stages |>
count(); return { total: total }`. The compiler should handle both
identically. The fact that it doesn't is an architecture flaw, not a
language design issue. The DSL should not force authors to manually
extract let bindings to work around a compiler limitation.

**Revised recommendation: expression extraction (lightweight Option A).**
Add a pre-lowering normalization pass that walks the body's statements
(including return expressions and record fields), finds sub-expressions
that need node creation (pipes, calls, service calls), lifts them into
synthetic `let` bindings prepended to the statement list, and replaces
them with identifiers.

After extraction:
```
// Before (what the DSL author writes):
return { total: stages |> count(), passed: passed }

// After extraction (what the lowerer sees):
let __pipe_0 = stages |> count()
return { total: __pipe_0, passed: passed }
```

Phase 1 handles `let __pipe_0 = stages |> count()` — it's a top-level
pipe chain, so it creates Collection nodes and registers the binding.
Phase 3 handles `return { total: __pipe_0 }` — it's an identifier
reference, so it wires to the Collection node's output.

This is:
- **Small**: one new function (`extract_nested_computation`)
- **Localized**: pre-processing step, doesn't change Phase 1 or Phase 3
- **Correct**: extracted form is semantically identical to the original
- **Aligned**: uses the existing architecture instead of fighting it
- **Incremental**: can extract one expression type at a time (pipes first,
  then calls, then service calls)

The expression types to extract (in priority order):
1. `Pipe` / `PipeCall` — triggers PipeOp fallback; Collection lowering exists
2. `Call` — triggers catch-all; callable endpoint creation exists
3. `ServiceCall` — triggers catch-all; transport triplet creation exists
4. `For` — triggers ForOp fallback; loop body expansion exists

After extracting these four, the only expressions remaining in return
position are: identifiers, field accesses, literals, and pure structural
operations (arithmetic, string interpolation, conditionals, records, lists).
These are exactly the forms that `resolve_return_expr_source` already
handles structurally. The `_` catch-all and `synthesize_expr_compute`
become unreachable.

Long-term, the Phase 1 / Phase 3 split is the real flaw — the lowerer
should be expression-recursive so extraction isn't needed. But expression
extraction gets to correctness without restructuring the compiler, and it
can be implemented incrementally.

After all causes are resolved (by any option), `synthesize_expr_compute` and
`synthesize_tagged_evaluator` have no callers and can be deleted, along with
`PrimitiveOpKind::ExprCompute`, `PrimitiveOpKind::PipeOp`, and
`PrimitiveOpKind::ForOp`. The `ExprComputeOp` in the resolver and the
`Passthrough` stubs in the emitter become dead code.

---

## Open Questions

1. **Which design option?** Options A (flatten), B (tree-recursive), and C
   (restrict language) all eliminate the wildcards. The choice is a language
   design decision: how much inline computation should the DSL support?
   This question must be answered before implementation begins.

2. **Pipe desugaring.** `x |> map(fn)` desugars to `map(fn, x)`. But `map`
   is a stdlib function resolved by the evaluator, not a DAG node. Structural
   pipe requires either: (a) a structural `Map` primitive that the lowerer
   emits directly, or (b) all pipe-target functions to be resolvable as DAG
   callables. This applies to Options A and B.

3. **Match arm bindings as structural inputs.** A `MatchDispatch` node needs
   to provide arm bindings to arm bodies. Currently the arm body is a lowered
   expression evaluated by the dispatch op. If arm bodies themselves need
   structural lowering (e.g., an arm body that calls a service), the
   `MatchDispatch` op needs to be a sub-DAG, not a flat node. This applies
   to Option B.

4. **Error quality.** When the lowerer rejects an expression (Invariant 7),
   the error must explain *why* and suggest alternatives. "Cannot lower
   `for x in list { body }` to structural DAG nodes" is not actionable.
   "Use a `stage` with `map` collection instead of inline `for`" is.

5. **Two wildcards, not one.** The `_` catch-all in
   `resolve_return_expr_source` (line ~10501) is the visible problem. But
   the `_ => {}` in Phase 1's statement loop (line ~4226) and in
   `collect_bound_callable_sources` (line ~12342) is the original cause.
   Both must be eliminated. Under any design option, the match in both
   locations must be exhaustive — every `Expr` variant gets an explicit arm.

6. **`Call` and `ServiceCall` in expression position.** These are already
   lowered by Phase 1 when they appear as top-level statements. Their
   presence in the Phase 3 catch-all means they appeared nested inside
   another expression (record field, conditional branch, pipe argument).
   Phase 1 never saw them because it only walks top-level statements.
   Under Option A, desugaring would extract them. Under Option B, the
   tree-recursive lowerer would create endpoint nodes inline. Under
   Option C, they would be disallowed in that position.
