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

### P2 — Type registry and type-shape inference fall back on unknown types

**Invariants**: #8 (correctness by construction), "no hacks or fallbacks"

Two earlier compiler layers still degrade unknown type information into lossy
stand-ins instead of failing the compile:

| Site | Function | Behavior |
|------|----------|----------|
| `src/00_foundation/ir/src/type_registry.rs` ~L1025 | `TypeRegistry::value_backing_for_type` | Prints warning and returns `ValueBacking::Json` |
| `src/00_foundation/ir/src/type_shape.rs` ~L216 | `infer_type_shape` | Prints warning and returns `TypeShape::Opaque("Unknown")` |

These are distinct from emit-time fallback. They change the compiler's own
internal understanding of the type before codegen or testgen even starts.

**Impact**: New or drifted DSL types silently collapse to generic JSON/opaque
shapes, which then propagates into testgen, emission, and runtime defaults.

**Fix direction**: Make unknown type registrations a hard diagnostic, or gate
these fallbacks behind an explicit compatibility mode that is off by default.

---

### P2 — Testgen fabricates `Json(Null)` mocks for unknown types

**Invariants**: #8 (correctness by construction), "no hacks or fallbacks"

`src/01_surfaces/codegen/src/testgen/codegen.rs` ~L7415 defaults any
DSL-defined product/coproduct type without an explicit mock entry to
`ValueExpr::Json(JsonValue::Null)`:

```rust
eprintln!(
    "warning: no explicit mock for type_id '{}'; using Json(Null) default",
    type_id
);
ValueExpr::Json(JsonValue::Null)
```

**Impact**: Generated tests continue with semantically invalid placeholder
inputs, so failures show up later as noisy runtime mismatches rather than as an
immediate unsupported-type error in test generation.

**Fix direction**: Make mock generation return `Result<ValueExpr, MockGenError>`
and require either an explicit mock or a structurally derived mock for every
reachable type.

---

### P2 — Additional invariant violation sites in mock/type pipeline

**Invariant**: "no hacks or fallbacks"

Sites not covered by the entries above:

| Site | Behavior |
|------|----------|
| `codegen/src/testgen/codegen.rs` ~L4916 | Port type lookup failure → defaults to `String` |
| `codegen/src/testgen/codegen.rs` ~L4823 | Lowering failure during testgen → silently uses unlowered analysis |
| `codegen/src/testgen/codegen.rs` ~L5721 | Effectful node with ExactOutputs → falls back to DryRun mode |
| `ir/src/type_shape.rs` ~L137,156,169,182,196 | Field/variant sub-DAG not found → `TypeShape::Opaque(...)` |

---

### P2 — Generated DAG test failure themes (1,019 failures observed 2026-03-09)

**Invariants**: "no hacks or fallbacks" — downstream consequence of the
type-registry, auto-mock, and type-shape fallbacks documented above.

The fallback chain `unknown type → ValueBacking::Json → Value::Json({"mock": true})`
produces structurally invalid mocks that crash at execution time. Five themes:

- **GetField on mock objects** (~530): `GetField 'comment'/'indent'/'spans'/'text'` on `{"mock": true}` — record-typed ports receive a mock without the expected fields
- **Unknown function in FnBody/MatchDispatch** (~132): `code_point`, `llm.Anthropic.Messages` — fn-body evaluator missing builtins or service op references
- **Field access on wrong value shape** (~55): fn bodies try `value.name` on mock values that lack the field
- **WrapScalar coercion** (~16): resource acquire nodes produce `{"mock": true}` instead of a list-coercible handle
- **Wrong output type** (~8): variant tests expect `Bool` but get a different shape from mock-driven execution

---

### P1 — Auto-testgen degrades compile failures into placeholder source files

**Invariants**: #8 (correctness by construction), "no hacks or fallbacks"

`src/01_surfaces/codegen/src/testgen_dag/dag_test_discovery.rs` currently
models compile failure as `AutoTestgenResult::Skipped { reason }`, and
`render_auto_testgen_for_module` converts that into a commented placeholder
Rust file instead of failing:

```rust
RenderedTestgenModule {
    content: format!(
        "// Auto-testgen skipped for '{}':\n{commented_reason}\n",
        module.module_name,
    ),
    path: output_path_for_module(output_dir, module),
}
```

The `gunbc-testgen` binary was fixed to fail closed instead of writing these
placeholders, but the helper remains and still encodes the fail-open behavior.

**Impact**: A caller can still treat an uncompilable module as a successful
testgen render, which masks the real compiler error and weakens test coverage.

**Fix direction**: Remove placeholder rendering entirely and make skipped
results unrepresentable at the rendering API boundary.

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

### P2 — `std/patterns.dag` breaks `make test-all` via testgen compile failure

**Invariants**: #8 (correctness by construction)

`std/patterns.dag` uses DSL features the lowerer does not yet support:

- Service calls inside `for` loops (`classify_files`, `read_text_files`, `read_binary_files`)
- Higher-order function parameters (`ensure`: `fn(T) -> Bool`)
- Generic type parameters used as values (`upsert`: `Resolve: -> R`, `transaction`: `Begin: -> R`)
- Associated output types (`Check.Output`)
- Pipe/for not yet structuralized in return expressions

The lowerer hits the P1 `eprintln!` + `continue` sites (line ~263 above),
producing a structurally incomplete DAG. Testgen then correctly fails closed
with `compile diagnostics: [VER004]: unwired required input`.

**Impact**: `make test-all` fails on main. The module is a design-spec file
expressing patterns that require future compiler work (FC-CF5, If-SubDag for
`[when]` on service calls, etc.).

**Fix direction**: Exclude `std/patterns.dag` from auto-testgen until the
lowerer supports the required features, or gate testgen discovery to skip
modules whose compilation produces diagnostics.

---

## What passed (2026-03-09 review)

- **Invariant 7**: `lower_expr` in `lib.rs` L10053–10449 — exhaustive match, no wildcard, returns `Result`. Same for `expr.rs` L268–442.
- **`extern func`**: Properly rejected at parse time (`src/03_source/daglang-syntax/src/parser.rs` L1465).
- **Phases 02–04, 06**: No `eprintln!`, no I/O side effects — clean pure functions.
- **I/O boundary** (Invariant 2): Only `08_materialize/transport/` performs direct I/O.
- **No backdoors**: Compiler provides metadata through output types, not callbacks.

---

## Structural prevention: `eprintln!` fallbacks are unenforceable

### Why this wasn't caught

Every fallback site uses `eprintln!("warning: ...")` followed by a default
value. This pattern is invisible to every enforcement mechanism in the repo:

- **`RUSTFLAGS="-D warnings"`** catches *compiler* warnings (unused vars,
  clippy). It has zero effect on runtime `eprintln!` output.
- **`cargo clippy`** cannot detect that an `eprintln!` inside a match arm
  represents a policy violation.
- **No test captures stderr.** Warnings flow to the terminal and disappear.
- **Generated tests don't validate mock preconditions.** They assert DAG
  execution results. If the mock is structurally wrong, the test crashes
  downstream in the executor — not at the point of the fallback.
- **One test validates the fallback as correct**: `identity_type_unknown_emits_
  name_verbatim` asserts the fallback behavior, treating it as intentional.

The gap: the invariant says "no hacks or fallbacks" but enforcement is zero.

### Fix: structured diagnostics that fail closed

Replace `eprintln!("warning: ...")` + default-value with a structured
diagnostic channel. Ban `eprintln!` via `disallowed_methods`.

**Three severity levels, two of which halt:**

- **`Diagnostic::Success`** — informational progress. Never fails. Replaces
  `eprintln!` used for status output (e.g., "dagbin cache hit").
- **`Diagnostic::Warning`** — **panics unconditionally.** If you think you
  need a warning, you have a bug. Every current `eprintln!("warning: ...")`
  site must become either a `Success` (if the behavior is correct) or an
  `Error` (if it's a fallback). There is no valid middle ground.
- **`Diagnostic::Error`** — propagates as `Err`, stops the pipeline phase.

**Why WARNING panics:** The entire class of bugs documented in this
postmortem exists because `eprintln!("warning: ...")` let code silently
degrade and continue. A warning that nobody reads is worse than no warning
— it creates the illusion of observability while masking failures. If a
condition is worth reporting, it's either normal (Success) or broken (Error).

**Enforcement:**

1. Add `eprintln` to `disallowed_methods` in the clippy policy. All current
   `eprintln!` sites must migrate to the diagnostic channel or get an
   explicit `#[allow]` with justification (binary entrypoints only).

2. `auto_mock_spec` (and callers) return `(MockSpec, Vec<Diagnostic>)`.
   Tests assert `diagnostics.is_empty()` for modules that should compile
   cleanly.

3. Each `eprintln!("warning: ...")` site becomes a forced decision: "is this
   actually an error I should fail on, or is it informational?" No more
   print-and-pray.

This converts the current "print and pray" pattern into a testable,
assertable contract. The 1,019 generated test failures would have shown up
immediately as diagnostic-count assertions at mock generation time, rather
than as downstream executor crashes.

### Root cause: `auto_mock_spec` doesn't receive the DSL type registry

The DSL type registry (`CompileOutput.dsl_type_registry`) contains every
type defined in `.dag` files — Format, Line, Span, FileEntry, FermiDepth,
etc. It is available at every call site that invokes `auto_mock_spec`.

But `auto_mock_spec` constructs its own static `TypeRegistry::with_core_
types()` singleton (auto_mock.rs:34), which only contains kernel + core
types. DSL-defined types are invisible to it.

The information exists and is compiled correctly. It just isn't passed to the
mock generator. A single parameter change (`auto_mock_spec(&dag, &name,
&registry)`) would give mock generation access to all structural type
information, enabling it to produce records with the correct fields instead
of `{"mock": true}`.

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
**Status:** Fixed in lowering; temporary lowering fallback remains.

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

Temporary containment added in this PR:

- Fail closed when callable arg wiring fails instead of logging and continuing.
- Fall back to a synthesized helper fn-body expression node for complex pure
  arg expressions when structural lowering is incomplete.

Removal trigger:

- Delete the helper-expression fallback once structural lowering can directly
  lower the relevant local/imported record-with-pipe arg shapes.

### P1 — dagbin cache reused stale lowered graphs across compiler changes

**Date:** 2026-03-09
**Status:** Fixed by bumping the cache epoch; kept here as a preventable case.

After lowering semantics changed, source-digest dagbin cache hits were still
treated as valid and reused previously lowered DAG shapes. That let an old,
structurally incomplete graph survive even after the compiler bug had been
fixed.

What made this preventable:

- The cache key only reflected source content, not compiler semantics.
- The failure showed up on a cache hit, which made the lowering fix appear
  ineffective until the cache epoch was bumped.
- The compiler had no invariant asserting that cached DAG shape/version matched
  the currently running lowerer.

Inspiration only:

- Treat compiler-semantics changes as cache-format changes by default.
- Add a targeted regression that exercises a changed lowering path through a
  warm cache and expects a rebuild.

### P1 — omitted repeatable CLI params collapsed to "missing" instead of `[]`

**Date:** 2026-03-09
**Status:** Fixed in CLI parsing; kept here as a preventable case.

`parse_generated_cli_args` omitted repeatable params entirely when the user
passed no values, instead of materializing the declared empty list. For
bootstrap, that meant `gitignore_categories` was absent rather than `[]`, which
changed downstream pure-fn behavior and contributed to the runtime failure.

What made this preventable:

- Repeatable params have a clear neutral element: the empty list.
- Generated CLI behavior and inline-evaluator behavior diverged on the same
  schema.
- The failure happened at the argument-materialization boundary, not in
  business logic.

Inspiration only:

- Treat repeatable params as present-with-empty-list even when omitted.
- Add cross-checks so generated CLI materialization and in-process execution
  share the same cardinality semantics.

### P2 — no structured artifact-hygiene pass let stale outputs accumulate

**Date:** 2026-03-09
**Status:** Open process gap.

We found multiple out-of-place generated or snapshot artifacts during cleanup:

- duplicate generated-test trees (`src/generated-tests/` and
  `src/10_test/generated-tests/...`)
- checked-in generated test sources that should have been ignored
- a tracked local snapshot artifact at `.dag-snapshots/workspace.json`

Small process change to add:

1. Inventory tracked generated/snapshot/temp paths with a scripted audit.
2. Classify each path as source-of-truth vs reproducible artifact.
3. For reproducible artifacts, add ignore rules before changing generator
   output paths.
4. Verify a fresh checkout still builds or regenerates cleanly after removals.
5. Add a CI check that fails when non-allowlisted generated/snapshot files are
   tracked.

### P2 — temporary scaffolding added during generated-test cleanup should be removed

**Date:** 2026-03-09
**Status:** Open removal list.

This cleanup PR introduced a few tactical compatibility/scaffolding changes.
They were reasonable for containment, but they should be explicitly removed
once the underlying paths are made principled:

- `daglang-lower`: synthesized `expr_value_*` fallback nodes for complex pure
  fn-call arguments. Remove once structural lowering handles those expressions
  directly.
- `gunbc-testgen`: binary-level fail-closed aggregation around
  `AutoTestgenResult::Skipped`. Remove the underlying placeholder-render path
  too; the rendering API should not encode "skipped but emit a file".
- `gunbc-tests`: tracked `src/lib.rs` + `build.rs` auto-discovery scaffold so
  the crate compiles without checked-in generated sources. Revisit once
  generated tests have a stable non-hacky home and inclusion model. In
  particular, stop glob-including every `src/generated/*.rs` file; the module
  index must be derived from the current testgen discovery set or the generator
  must remove stale outputs on rename/delete.
- dagbin cache: manual cache-epoch bump to flush stale lowered graphs after the
  lowering fix. Replace with a more principled cache key/versioning strategy.
