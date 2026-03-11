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

### P2 — Generated DAG test failure themes (1,019→888 after fix, observed 2026-03-09)

**Invariants**: "no hacks or fallbacks" — downstream consequence of the
type-registry, auto-mock, and type-shape fallbacks documented above.

**Fixed (themes 1, 3, 4):** The fallback chain
`unknown type → ValueBacking::Json → Value::Json({"mock": true})` produced
structurally invalid mocks. Threading the DSL type registry into
`auto_mock_spec` and adding `product_witness()` for record types eliminated
all `GetField on {"mock": true}` failures, WrapScalar coercion failures,
and field-access-on-wrong-shape failures. 1,019 → 888 failures.

**Remaining (888 failures):** Three FnBody evaluator gaps documented below.

---

### P2 — Builtin callable contracts have no evaluator implementation (584→0 after fix)

**Invariant**: #8 (correctness by construction)

The typechecker registered `builtin_callable_contracts()` — a manually
maintained `Vec<(String, CallableContract)>` of standalone functions that
pass typecheck but have no definition as DSL `fn` items.

**Dead entry cleanup (2026-03-09):** 10 of 14 standalone entries had zero
call sites in any .dag file and were removed: `render_cytoscape_html`,
`render_mermaid_markdown`, `render_test_listings`, `render_graph_structure`,
`render_source_artifacts`, `compute_topology_diff`, `render_annotated_mermaid`,
`detect_runtime`, `generate`, `now`. 4 remain with actual callers: `eq`,
`chars`, `code_point`, `build_token`.

These builtins existed in **three** places with **different implementations**:

| Layer | Mechanism | Status |
|-------|-----------|--------|
| Typecheck | `CallableContract` in `builtin_callable_contracts()` | Accepts the call |
| Lowerer | `LoweredExpr::Call { name, args }` — passthrough, no validation | Produces IR |
| Emit | Special-case codegen in `fn_codegen.rs` (e.g., `code_point` → `code_point_i64()`, `chars` → `.chars()`) | Works for compiled binaries |
| Evaluator | Not handled in `eval_call()` | **Was: `unknown function: {name}`** |

**Fixed (2026-03-09):** Added `code_point` and `chars` implementations to
`eval_call()` and `eval_pipe_method()` in `daglang-lower/src/eval.rs`:

- `code_point(c: Char) -> Int`: returns Unicode scalar value (`c as i64`)
- `chars(s: String) -> List<Char>`: splits string into single-char strings
- `PipeMethod::Chars`: same as standalone `chars`, moved out of the
  delegate-to-sibling-fn fallback group

This eliminated all 591 `unknown function: code_point/chars` failures.
The fix unmasked the next layer: those fn bodies now get one step further
and fail on `unbound variable: zero_width_codepoints` (data declaration
gap, documented below).

**Open question:** The remaining 4 builtins (`eq`, `chars`, `code_point`,
`build_token`) are still a manually maintained list with no structural
binding between typecheck, emit, and eval. The three-layer disconnect
persists — see "Manually maintained registries" section below.

---

### P2 — Data declarations invisible to fn-body evaluator (658→~95 after fix)

**Invariant**: #8 (correctness by construction)

DSL `data` declarations (compile-time constants) are evaluated by
`build_data_values()` at lowering time and stored in
`LowerOutput.data_values`. They are consumed during **DAG wiring** — when a
service call argument references a data ident, the lowerer creates a literal
source node.

But fn-body evaluation ran at **runtime** via `evaluate_fn_body()`, which
received only `inputs: HashMap<String, Value>` and `sibling_fns`. The
evaluator's `Env` had no mechanism to look up data declarations.

**Fixed (2026-03-09):** Added `evaluate_fn_body_with_data()` in
`daglang-lower/src/eval.rs` that accepts `data_values: &HashMap<String,
serde_json::Value>` and seeds the evaluator's `Env` with converted values
before executing statements. Added `json_to_eval_value()` for recursive
`serde_json::Value` → `Value` conversion (objects → `Value::Map`, arrays →
`Value::List`).

Threading path: `LowerOutput.data_values` → `CompileLoweredResult.data_values`
→ `CachedCompileData.data_values` (with `#[serde(default)]` for cache
compat) → `resolve_lowered_dag_with_data()` → `resolve_node_body()` →
`resolve_op()` → `resolve_domain()` → `FnBodyCallableOp.data_values` →
`evaluate_fn_body_with_data()`.

Cache version bumped from 3 → 4 to invalidate stale cached DAGs without
`data_values`.

**Result:** 658 → ~95 failures. Most data declaration references now resolve.
~38 remaining `zero_width_codepoints` failures are in generated tests that
construct their own mocks and DAGs (bypassing `build_dsl_graph`), so
`data_values` doesn't reach them through the test execution path. ~57
remaining `unbound variable` failures are in other data declarations
(`items`, `entries`, `sections`, `lines`, `stages`, `categories`) where the
data either isn't propagated through a specific test path or the ident uses
a qualified name not matching the `data_values` key.

---

### P2 — Fn-body evaluator produces placeholder strings for DSL types (15 failures)

**Invariant**: #8 (correctness by construction)

When the fn-body evaluator runs a fn node in DryRun mode, the DAG executor
provides mock input values for the fn's parameters. For DSL-defined record
types that the `product_witness` fix doesn't reach (because the mock is
generated by a different path — the fn-body node's own parameter mocking
rather than `auto_mock_spec`), the fallback is still
`Value::Str("<TypeName>")`.

Affected types: `CommentSyntax` (8), `GitignoreFile` (3),
`GitignoreCategory` (2), `Summary` (2).

This is the same root cause as the themes 1-3 fix but in a different code
path: the fn-body node's parameter mock values are constructed without the
DSL type registry, so they fall back to placeholder strings.

**Fix direction**: Ensure all paths that construct mock values for fn-body
parameters use the same `product_witness` logic with the DSL type registry.

---

### P2 — Service call names leak into fn-body evaluation via MatchDispatch (72 failures)

**Invariant**: #8 (correctness by construction)

`tools_design` uses match/dispatch patterns where the lowerer creates
`MatchDispatch` nodes that reference service call qualified names (e.g.,
`llm.Anthropic.Messages`). When these nodes execute in DryRun/test mode,
the fn-body evaluator tries to resolve `llm.Anthropic.Messages` as a
function call, producing `unknown function: llm.Anthropic.Messages`.

The error path: `MatchDispatch` node → `eval_call("llm.Anthropic.Messages")`
→ not a sibling fn, not a builtin, not uppercase → `unknown function` error.

This affects `tools_design` (69 failures) and `tools_readme` (3 failures) —
modules that use service-call-based match dispatch patterns.

**Fix direction**: `MatchDispatch` nodes in DryRun mode should produce a
default/mock value for the matched service call rather than trying to
evaluate the service call name as a function.

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

### Root cause and fix: `auto_mock_spec` lacked DSL type registry + product witness generation

**Root cause 1 — missing registry (fixed 2026-03-09):**

The DSL type registry (`CompileOutput.dsl_type_registry`) contains every
type defined in `.dag` files — Format, Line, Span, FileEntry, FermiDepth,
etc. It is available at every call site that invokes `auto_mock_spec`.

But `auto_mock_spec` constructed its own static `TypeRegistry::with_core_
types()` singleton, which only contained kernel + core types. DSL-defined
types were invisible to it.

Fix: added `dsl_registry: Option<&TypeRegistry>` parameter to
`auto_mock_spec`. All call sites updated — compile-time callers pass
`Some(&result.dsl_type_registry)`, generated test code rebuilds the graph
and passes the registry at test runtime.

**Root cause 2 — no product witness generation (fixed 2026-03-09):**

Even with the DSL type registry available, `typed_witness_value` could not
produce structural mock values for product (record) types. The path was:
`resolve_type("ConfigFormat")` → type DAG with `Product` node →
`witnesses_checked()` → `scalar_witness_for_base("ConfigFormat")` →
`Value::Str("<ConfigFormat>")` (placeholder) → filtered out → fallback to
`ValueBacking::Json` → `{"mock": true}`.

Fix: added `product_witness()` in `auto_mock.rs` that detects `Product`
type DAGs, extracts field names and their SubDag type DAGs, and recursively
builds `Value::Map` with field-level witnesses (depth-limited to 4).

**Progression:** 1,013 → 888 → 1,165 (more test modules) → 791 (after all fixes).

Current error distribution (791 remaining):

| Category | Count | Root cause |
|----------|-------|------------|
| `unknown function: llm.*` | 72 | Service call names leaking into fn-body eval via MatchDispatch |
| `unbound variable` (residual data) | ~95 | Data decls not reaching generated test paths |
| `WrapScalar` / `{"mock": true}` | ~67 | Resource handle mocks in different code path than product_witness |
| `no match arm matched` | 10 | Sum type placeholder strings don't match variant arms |
| Other downstream cascades | ~547 | Effects of above |

These are all DSL evaluator completeness gaps, not mock generation issues.

---

## Scenario backlog

### P1 — Shared fn nodes caused wrong data flow across entrypoints

**Date:** 2026-03-09
**Status:** Fixed in lowering (`daglang-lower/src/lib.rs`).

`gist-recent` produced an empty gist and dumped a 599K-line diff to the
console as a boundary output. `gist-diff` was unaffected.

**What happened:**

`gist.dag` defines three `func` entrypoints (`gist`, `gist_diff`,
`gist_recent`) and a shared `fn render_diff_markdown(diff, branch, base_ref)`.
Both `gist_diff` and `gist_recent` call `render_diff_markdown`.

The lowerer creates a single DAG node for each `fn` item. When multiple
`func` callables reference the same `fn`, the lowerer wires data edges from
caller-specific transport outputs to the shared fn node's input ports. A
`has_edge_to_port` guard prevents duplicate edges to the same port — but
this means only the **first** caller's edges are wired. The second caller's
transport outputs remain unconnected.

At runtime with entrypoint slicing for `gist_recent`:

1. The shared `render_diff_markdown` node received inputs from `gist_diff`'s
   context (unsuffixed Diff transport, CurrentBranch_c1, gist_diff's
   base_ref param) — wrong data.
2. `gist_recent`'s Diff_c1 transport parse output (`diff`) had no downstream
   data edge, so `detect_boundaries` flagged it as a boundary output. The
   executor printed the entire 599K-line diff to the console.
3. The gist content was empty because `render_diff_markdown` received
   gist_diff's mock/empty inputs, not gist_recent's actual diff.

A secondary issue compounded the problem: `add_callable_nodes` creates
`__deps` ordering edges from all called fn endpoints to the caller's target
using the global `endpoints_by_name` map, which always points to the
original fn node. This meant the original `render_diff_markdown` was
backward-reachable from `gist_recent` via `__deps`, pulling `gist_diff`'s
entire transport subgraph into the sliced DAG (7 extra nodes).

**Fix — fn node cloning with `__deps` redirect:**

Two changes in `add_service_call_edges` (`lib.rs` lines 7741–7821):

1. **Fn node cloning:** Track per-fn-node usage across callables via
   `fn_node_use_count`. When a fn_body node is referenced by a second
   caller, clone it with `_fc{N}` suffix (e.g., `render_diff_markdown_fc1`).
   Only fn items (with `fn_body: Some(..)`) are cloned — func items use
   passthrough wiring (`__out:` ports) that doesn't carry over to clones.
   Update `bound_callable_sources` and `fn_name_overrides` to point to
   the clone, so `wire_fn_call_arguments` and service call arg wiring
   resolve to the correct per-caller copy.

2. **`__deps` edge redirect:** After cloning, find the `__deps` edge from
   the original fn node to the current callable's target and redirect
   `from_node` to the clone. This prevents backward reachability from
   pulling in the wrong caller's transport subgraph during entrypoint
   slicing.

This mirrors the existing `clone_transport_triplet` pattern for service
call transport nodes.

**Result:**

| Metric | Before | After |
|--------|--------|-------|
| gist-recent sliced nodes | 42 (7 from gist_diff) | 35 (clean) |
| gist-recent progress | 42/43 done, 1 skipped | 36/36 done, 0 skipped |
| Boundary diff dump | 599K lines on console | Gone |
| All workspace tests | Pass | Pass |

**What made this preventable:**

- The `has_edge_to_port` guard was designed for a world where each fn node
  is called from exactly one entrypoint context. The guard silently
  succeeded (returned early) instead of failing loudly when a second caller
  tried to wire the same port.
- The `__deps` edges were created in `add_callable_nodes` using a global
  endpoint map, then never revisited when fn cloning happened in a later
  pass (`add_service_call_edges`). The two passes didn't coordinate.
- No test exercised a `.dag` file with multiple `func` entrypoints calling
  the same `fn` item and then slicing to the second caller.

---

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

---

### P2 — Manually maintained registries with no implementation binding

**Date:** 2026-03-09
**Invariant**: #8 (correctness by construction), "no hacks or fallbacks"

The codebase has multiple manually maintained registries that map names to
type-level contracts without any structural connection to implementations.
Entries can be added or removed without the compiler detecting missing or
stale implementations.

**Inventory of manually maintained registries:**

| Registry | Location | Entries | What it maps |
|----------|----------|---------|-------------|
| `PIPE_METHOD_REGISTRY` | `daglang-syntax/src/lib.rs:754` | 21 | method name → PipeMethodDef (arity, params, output type, collection op) |
| `builtin_callable_contracts` | `daglang-typecheck/src/lib.rs:2120` | 4 standalone + 21 derived from PIPE_METHOD_REGISTRY | function name → CallableContract (arity, params, output type) |
| Language models (Rust/Go/C) | `daglang-emit/src/language_model.rs:175+` | ~90 total | DSL type name → target language syntax |
| Runner/integration catalogs | `ir/src/transport/github_actions.rs` | 12 | integration/runner ID → CI/CD definition |
| `CODEGEN_INPUT_GLOBS` | `ir/src/resource/defs.rs:16` | 4 | hardcoded file paths/globs for codegen dependency tracking |

**Derived (not manually maintained) — for contrast:**

| Registry | Location | What it maps |
|----------|----------|-------------|
| `collect_unique_callables` | `daglang-typecheck/src/lib.rs:2001` | Walks AST: FnDef, FuncDef, PatternDef, TypeDef → CallableContract |
| `collect_service_call_contracts` | `daglang-typecheck/src/lib.rs:2274` | Walks AST: ServiceDef → ServiceCallContract |
| `STANDARD_SYMBOLS`, `ANSI_MAPPINGS`, Unicode blocks | `ir/src/generated/mod.rs` | Generated from DSL compilation |

**The three-layer disconnect (builtin_callable_contracts):**

The same function name must be handled independently in three places:

1. **Typecheck** (`builtin_callable_contracts`): registers name + type signature
2. **Emit** (`fn_codegen.rs` special cases): generates target-language code
3. **Eval** (`eval_call()` match arms): interprets at runtime

No shared registry or compile-time check enforces that all three agree.
Only `code_point` and `chars` have emit handlers. None have eval handlers.
Adding a typecheck entry without an eval handler is undetectable until
runtime crashes.

**Dead entry cleanup (2026-03-09):** Removed 10 standalone entries from
`builtin_callable_contracts` that had zero .dag call sites:
`render_cytoscape_html`, `render_mermaid_markdown`, `render_test_listings`,
`render_graph_structure`, `render_source_artifacts`, `compute_topology_diff`,
`render_annotated_mermaid`, `detect_runtime`, `generate`, `now`. Updated
tests in `daglang-typecheck/src/tests.rs` and `daglang-cli/src/compile/tests.rs`.

**`or_insert` semantics:** `collect_unique_callables` appends builtins after
AST-derived callables using `entry(name).or_insert(Some(contract))`. A DSL
`fn` definition silently shadows the builtin — the role is ambiguous (fallback?
forward declaration? spec?).

**`allow_unresolved_imports`:** A `TypecheckOptions` flag that when true,
suppresses `UnresolvedCallTarget` errors. Used in several utility paths in
`daglang-driver`. This means in permissive mode, ANY unresolved call is
silently accepted — not just builtins.

**Open question:** Whether the remaining 4 builtins (`eq`, `chars`,
`code_point`, `build_token`) should be `extern func` declarations in .dag
files rather than hardcoded in Rust. This would make the typecheck registry
fully derived from source, but requires that `extern func` doesn't introduce
heuristic-based resolution.

---

# Architectural analysis: representation convergence and the interpreter/compiler fork

**Date:** 2026-03-09
**Status:** Analysis only — no code changes.

## Unifying diagnosis

Every item in this postmortem is an instance of the same architectural
violation: **a semantic concept has multiple independent representations, and
no structural mechanism enforces that all representations agree.**

| Postmortem item | Concept with multiple representations |
|-----------------|--------------------------------------|
| Registry disconnect | Builtins defined independently in typecheck, emit, eval |
| Silent `eprintln` fallbacks | "What to do with unknown types" answered independently by type_registry, type_shape, auto_mock, emit |
| `[when]` guard dropping | Guard syntax handled by fn-body path, pattern expansion path, but not func-body service call path |
| Data decls invisible to evaluator | Data declarations flow through DAG wiring but not eval scope |
| Shared fn nodes / wrong data flow | One fn node representation shared by multiple callers with no per-caller isolation |
| Cache staleness | "Is this cached artifact valid?" answered by source digest but not compiler semantics |
| Builtin callable contracts | Same function name handled independently in typecheck, emit, eval with no shared binding |

The fix is not "add more tests" or "add more lints." It is **representation
convergence**: each concept gets exactly one authoritative representation
early in the pipeline, and all downstream consumers are structurally forced
to handle all cases (via exhaustive enum match, trait implementation, or
pipeline derivation).

---

## The builtins are not special

The 4 remaining builtins (`eq`, `chars`, `code_point`, `build_token`) are
not intrinsically host-bound. They are trivially expressible in the DSL as
it exists today:

| Builtin | DSL equivalent | Why it was Rust |
|---------|---------------|-----------------|
| `eq(a, b)` | `a == b` (DSL has `==` operator) | Predates DSL operator support |
| `chars(s)` | `s \|> chars()` (already a pipe method) | Standalone call form predates pipe method |
| `code_point(c)` | Identity — `Char` is `Int where brand("Char")` | Predates refined type definitions |
| `build_token(...)` | Record constructor `{ token: payload, scheme: scheme, expires_at: None }` | Predates DSL record literals |

These are **workarounds that outlived the conditions that created them.** The
DSL grew past them, but the Rust implementations persisted. The three-layer
registry disconnect is scar tissue from that temporal gap, not an inherent
architectural need.

This pattern — temporary Rust bypass that persists after the DSL becomes
capable — recurs across the postmortem. The `eprintln` fallbacks, the
`ExprCompute` bindings (since deleted), and the resolver ops are all
instances.

---

## The compiler and the interpreter are different things

The compiler pipeline (stages 03–07) produces `VerifiedDag<LoweredOp>`.
That is its product. The compiler's job is done.

Running that product is a separate concern — an **interpreter**. The
relationship is the same as `rustc` producing a binary and the OS executing
it. The compiler produces an artifact; the interpreter consumes it.

Today the codebase does not make this separation. The interpreter is
smeared across three places, none of which are called "the interpreter":

```
Compiler (stages 03–07):
  .dag → parse → resolve imports → typecheck → lower → VerifiedDag<LoweredOp>
                                                              ↓
                                                        [THE ARTIFACT]
                                                              ↓
Interpreter (currently: stages 08–09, entangled):
  eval.rs (in daglang-lower)     — actual interpretation logic
  gunbc-resolve                  — rewraps eval.rs + reimplements half of it
  gunbc-primitives               — third implementation of same operations
  gunbc-exec                     — DAG scheduler (calls Executable::execute)
  gunbc-lib-transport            — I/O (shell, HTTP, filesystem)
```

The interpreter needs exactly two capabilities:
1. **Evaluate pure operations** — field access, binary ops, string
   interpolation, fn bodies, pattern matching, collection ops. This is
   what `eval.rs` does.
2. **Execute transport** — shell commands, HTTP requests, file I/O. This
   is what `gunbc-lib-transport` does.

Everything else is translation glue that exists because the interpreter
was never designed as a coherent unit.

---

## Three dissonances from the missing interpreter boundary

### 1. The evaluator lives inside the compiler

`eval.rs` is inside `daglang-lower` (stage 05, "Graph Lowering"). It is
not a lowerer — it is the interpreter's core. It serves two callers:

- **Compile-time:** `build_data_values()` evaluates `data` declarations
  during lowering. This is constant folding — a legitimate compiler concern.
- **Runtime:** `FnBodyCallableOp` in the resolver calls
  `evaluate_fn_body_with_data()` during DAG execution. This is
  interpretation.

The lowerer depends on the evaluator for constant folding. Fine — the
compiler can call the interpreter as a library (like `rustc` calling MIRI
for const eval). But the evaluator should be its own crate that both the
compiler and the interpreter depend on, not something trapped inside the
compiler.

### 2. The resolver reimplements the evaluator

The resolver wraps each `LoweredOp::Primitive` variant in a separate Rust
struct implementing `Executable`. Most of these structs either delegate to
the evaluator or reimplement the same logic independently:

| Resolver op | What it does | Evaluator equivalent |
|---|---|---|
| `GetFieldOp` | Extract field from map | `eval_expr(FieldAccess)` |
| `StringInterpolateOp` | Concat strings | `eval_expr(StringInterpolation)` |
| `BinaryOpNode` | `a + b`, `a == b`, etc. | `eval_expr(BinaryOp)` |
| `ConditionalOp` | if/else | `eval_expr(IfElse)` |
| `MatchDispatchOp` | pattern match | Delegates to `eval_match()` |
| `RecordConstructOp` | build a map | `eval_expr(Record)` |
| `ListConstructOp` | build a list | `eval_expr(List)` |
| `FnBodyCallableOp` | fn body eval | Delegates to `evaluate_fn_body_with_data()` |

The resolver exists because the executor expects `DynOp` (trait objects)
and the compiler produces `LoweredOp` (data enums). Someone had to bridge
the gap. Instead of building a clean interpreter, the bridge reimplemented
half the evaluator. Two implementations of the same semantics, neither
aware of the other.

### 3. `gunbc-primitives` is a third implementation

`PrimitiveOp` in `gunbc-primitives` defines Map, Filter, Fold, Parse,
Extract, Format, etc. The evaluator in `eval.rs` implements
`eval_pipe_method()` with Map, Filter, Fold, etc. The resolver implements
`CollectionOp` variants. Three places where "map over a list" is defined.

---

## Root cause: the interpreter was never designed

The compiler pipeline (stages 03–07) was designed intentionally. Each stage
has a clear purpose: parse, resolve imports, typecheck, lower to IR, derive
metadata, emit code. Clean, well-scoped.

The interpreter was not designed. It accreted:

- The evaluator went into the lowerer (because it needed `LoweredExpr`
  types, which live there).
- The resolver wrapped each lowered op in an `Executable` trait impl
  (because the executor expected trait objects, not data enums).
- `gunbc-primitives` predated both and was never reconciled.

Each was a reasonable local decision. Together they created a system where
the same semantics are implemented three times with no structural binding —
the exact pattern that produces every other item in this postmortem.

---

## The interpreter as a first-class concept

The interpreter should be a crate (or small group of crates) that:

1. Takes `VerifiedDag<LoweredOp>` as input — the compiler's artifact.
2. Walks nodes in topological order (the scheduler).
3. For each node, dispatches to exactly one of:
   - The evaluator (pure operations) — one implementation, no duplication.
   - The transport layer (I/O operations) — shell, HTTP, filesystem.
4. Returns execution results.

No resolver. No `LoweredOp → DynOp` translation. No 30 separate op structs
wrapping evaluator calls. The lowered IR IS the executable representation.
The interpreter reads it directly.

```
Target architecture:

Compiler:                         Interpreter:
  daglang-syntax                    daglang-eval (pure evaluation)
  daglang-resolve                       ↓
  daglang-typecheck                 gunbc-interp (scheduler + dispatch)
  daglang-lower ←(const eval)→          ↓
  daglang-derive                    gunbc-lib-transport (I/O)
  daglang-emit (Branch A only)
       ↓
  daglang-driver (orchestrator)
       ↓
  VerifiedDag<LoweredOp> ────────→ gunbc-interp
```

The compiler and interpreter share two things:
- `gunbc-ir` — the DAG/Node/Port/Edge/Value types (the artifact format).
- `daglang-eval` — the evaluator (compiler uses it for const folding,
  interpreter uses it for runtime evaluation).

They share nothing else. The compiler doesn't know about transport. The
interpreter doesn't know about parsing or typechecking.

---

## How this resolves postmortem items

- **Registry disconnect:** Eliminated. No builtins — everything is DSL `fn`
  evaluated by one evaluator in `daglang-eval`. The three-layer disconnect
  (typecheck / emit / eval) collapses to: typecheck derives from AST,
  evaluator interprets the lowered body.

- **Silent fallbacks:** The type_registry / type_shape / auto_mock / emit
  fallbacks exist because each layer independently handles unknown types.
  One evaluator with one type understanding = unknown is a hard error at the
  first gate.

- **`[when]` guard dropping:** The resolver's service ops don't handle
  guards because they operate independently of the evaluator. One evaluation
  path in the interpreter = guards always evaluated.

- **Data declarations invisible to evaluator:** The evaluator IS the
  execution path, so data declarations are always in scope.

- **Shared fn nodes:** The resolver clones fn nodes because it reifies them
  as separate `Executable` instances. In the interpreter, fn evaluation is
  a function call to `daglang-eval` — no node cloning needed.

- **Cache staleness:** Fewer moving parts = fewer cache invalidation
  concerns. The interpreter evaluates `LoweredOp` directly, so the cache
  key only needs to cover source + lowerer version.

---

# Refactor plan: pipeline convergence

**Date:** 2026-03-09
**Status:** COMPLETE. All 8 phases executed (2026-03-09 to 2026-03-10).
**Goal:** Separate the compiler from the interpreter. Eliminate the
three-way semantic duplication (evaluator / resolver / primitives).

**Execution log:**

| Phase | Commit | Summary |
|-------|--------|---------|
| 0a | `6e237e8f` | 4 hardcoded builtin callables → DSL fn items |
| 0b | `5bc3ed78` | Pipe methods use string dispatch (PIPE_METHOD_REGISTRY) |
| 1 | `617b28b9` | 10 pure-value op structs → single PurePrimitiveOp dispatch |
| 2 | `2e660ff3` | Evaluator delegation + data declaration embed in DAG |
| 3 | `bed0762e` | 8 transport-specific service op structs → 2 parameterized |
| 4 | `29826fc3` | Collapse FnBodyCallableOp + DeclaredOutputCallableOp → CallableOp |
| 5 | `10893eb1` | Delete gunbc-primitives crate |
| 6 | `9473fc79` | Extract evaluator to daglang-eval crate |
| 7 | `db536a0e` | Create gunbc-interp crate |
| 8 | `4b3296c6` | Delete PipeMethod enum, simplify pipe registry |
| — | `797390f4` | Delete pipe operator (`\|>`) from compiler and DSL |
| — | `7b5713aa` | Fix pipe→call evaluation, lint, make test-all |

**Follow-on (2026-03-10):** Mock type fix + type registry cleanup reduced
generated test failures from 529 → 115 (see §7 in fail-closed gaps above).
Deleted `tools.design` (dead SDLC code, 89 pre-existing failures).
Deleted 39 redundant Rust type registrations (source of truth is .dag).
Deleted 13 container alias type registrations (migrated port type IDs to
parametric forms). Fixed inline record parsing, registration, and lookup.
Fixed `Value::Enum` consistency in variant witnesses.

**Target architecture:** See `src/ARCHITECTURE.md` for the full target
state, stage contracts, and rationale.

---

## Before and after

### BEFORE — current pipeline

```
COMPILER (stages 03–07):
  03  daglang-syntax         parse .dag → AST
  03  daglang-resolve        discover imports → ModuleGraph
  04  daglang-typecheck      validate types → TypedProject
  05  daglang-lower          lower to graph IR → VerifiedDag<LoweredOp>
                             (ALSO contains eval.rs — the interpreter core)
  06  daglang-derive         extract metadata → DerivedArtifacts
  07  daglang-emit           generate Rust/Go/C → EmissionBundle
  02  daglang-driver         orchestrate all of the above → CompileOutput

INTERPRETER (stages 08–09, entangled with compiler):
  08  gunbc-primitives       PrimitiveOp enum — reimplements eval ops
  08  gunbc-resolve          LoweredOp→DynOp — reimplements eval ops AGAIN
  08  gunbc-lib-transport    shell/HTTP/filesystem I/O
  08  gunbc-lib-blob         blob content acquisition
  09  gunbc-exec             DAG scheduler — calls Executable::execute()

TEST:
  10  gunbc-test             mock synthesis, Mockable trait
  10  gunbc-testgen-registry target registry
  10  gunbc-tests            auto-generated tests
```

**Problems:**

1. `eval.rs` (the interpreter's brain) is inside the compiler
   (`daglang-lower`). The compiler is being used as a runtime library.

2. `gunbc-resolve` reimplements pure operations that `eval.rs` already
   handles. 10 separate op structs that duplicate evaluator logic. Exists
   solely to bridge `LoweredOp` (data) to `DynOp` (trait objects).

3. `gunbc-primitives` is a third implementation of the same operations.
   Map, Filter, Fold, etc. defined independently from both eval.rs and
   the resolver.

4. `gunbc-resolve` has 9 workspace deps — more than any other crate.
   It depends on the compiler driver, the lowerer, the deriver, transport,
   primitives, blob, exec, infra, and ir. This is the coupling symptom of
   a missing architectural boundary.

5. The compiler and interpreter share concerns they shouldn't. The
   compiler produces `VerifiedDag<LoweredOp>`. That's its product. But
   because `eval.rs` lives inside the compiler, the interpreter depends
   on compiler internals. And because the resolver depends on the driver,
   the interpreter transitively depends on the emitter — which it never
   uses.

### AFTER — separated pipeline

```
SHARED FOUNDATION:
  00  gunbc-infra            hashing, resource IDs, manifests
  00  gunbc-ir               Dag, Node, Port, Edge, Value, types
  00  daglang-contract       Verdict, Diagnostic, spans

COMPILER (produces the artifact):
  03  daglang-syntax         parse .dag → AST
  03  daglang-resolve        discover imports → ModuleGraph
  04  daglang-typecheck      validate types → TypedProject
  05  daglang-expr           LoweredExpr, LoweredFnBody types (IR for exprs)
  05  daglang-lower          lower to graph IR → VerifiedDag<LoweredOp>
                             (calls daglang-eval for const folding only)
  06  daglang-derive         extract metadata → DerivedArtifacts
  07  daglang-emit           generate Rust/Go/C → EmissionBundle
  02  daglang-driver         orchestrate all of the above → CompileOutput

EVALUATOR (shared by compiler + interpreter):
  05  daglang-eval           evaluate LoweredExpr — one implementation
                             deps: gunbc-ir, daglang-expr

INTERPRETER (consumes the artifact):
  08  gunbc-interp           scheduler + dispatch on LoweredOp
                             pure ops → daglang-eval
                             I/O ops → gunbc-lib-transport
                             deps: gunbc-ir, daglang-eval, transport
  08  gunbc-lib-transport    shell/HTTP/filesystem I/O (unchanged)
  08  gunbc-lib-blob         blob content acquisition (unchanged)

TEST:
  10  gunbc-test             mock synthesis (no primitives dep)
  10  gunbc-testgen-registry target registry
  10  gunbc-tests            auto-generated tests
```

**Deleted crates:**
- `gunbc-primitives` — operations consolidated into `daglang-eval`.
- `gunbc-resolve` — resolution layer eliminated. Transport spec wiring
  folded into `gunbc-interp` or `gunbc-lib-transport`.

**New crates (from moved code, not new logic):**
- `daglang-expr` — expression IR types extracted from `daglang-lower`.
- `daglang-eval` — evaluator extracted from `daglang-lower`.
- `gunbc-interp` — clean interpreter, replaces `gunbc-resolve` + the
  scheduling role of `gunbc-exec`.

**What each change buys:**

| Change | What it eliminates | What it enables |
|--------|-------------------|-----------------|
| Extract `daglang-eval` | eval.rs trapped in compiler; resolver reimplements it | One implementation of every pure operation; compiler calls eval for const folding without being the eval host |
| Extract `daglang-expr` | Expression types coupled to lowerer crate | Evaluator has its own leaf types; no dependency on compiler internals |
| Create `gunbc-interp` | Resolver's 30 op structs wrapping evaluator calls; `LoweredOp→DynOp` translation | Direct dispatch on `LoweredOp`: pure→eval, transport→I/O; no type-erasure overhead |
| Delete `gunbc-primitives` | Third implementation of Map/Filter/Fold/etc. | One implementation of collection ops in `daglang-eval` |
| Delete `gunbc-resolve` | 9 workspace deps, reimplemented evaluator, `FnBodyCallableOp` delegation | Clean interpreter with ~4 deps (ir, eval, transport, blob) |
| Rename/slim `gunbc-exec` | Executor coupled to `DynOp` trait objects | Scheduler is generic; `gunbc-interp` provides the `LoweredOp` dispatch |

**Dependency graph comparison:**

```
BEFORE (gunbc-resolve):              AFTER (gunbc-interp):
  daglang-derive                       gunbc-ir
  daglang-driver                       daglang-eval
  daglang-lower                        gunbc-lib-transport
  gunbc-exec                           gunbc-lib-blob
  gunbc-infra
  gunbc-ir
  gunbc-lib-blob
  gunbc-lib-transport
  gunbc-primitives
  ─────────────────                    ─────────────────
  9 workspace deps                     4 workspace deps
```

The interpreter no longer depends on the compiler. The compiler no longer
hosts the interpreter's core logic. Each can evolve independently.

---

## Guiding constraints

1. **Every phase compiles and passes `cargo test --workspace`.** No phase
   leaves the codebase in a broken state.
2. **No phase changes more than one boundary at a time.** Crate splits,
   crate deletions, and semantic changes happen in separate phases.
3. **Deletion requires proof.** Before deleting code, a previous phase must
   have made it unreachable or redundant and verified that with tests.
4. **The interpreter stays thin.** Its job is dispatch: pure → eval,
   transport → I/O. No business logic in the interpreter itself.

---

## Current dependency graph (relevant subset)

```
daglang-syntax          (leaf)
daglang-contract        (leaf)
gunbc-infra             (leaf)
    ↓
gunbc-ir                (infra, contract, delegate-macros)
    ↓
gunbc-exec              (ir)
    ↓
gunbc-primitives        (infra, ir, exec)
gunbc-lib-transport     (ir, exec)
    ↓
daglang-lower           (contract, syntax, typecheck, ir)
    ↓
daglang-derive          (lower, contract, ir)
daglang-emit            (derive, lower, syntax, typecheck, ir)
    ↓
daglang-driver          (contract, syntax, resolve, typecheck, lower, derive, emit, ir)
    ↓
gunbc-resolve           (derive, driver, lower, exec, infra, ir, blob, transport, primitives)
    ↓
gunbc-codegen           (resolve, + nearly everything)
gunbc-test              (ir, exec, primitives)
```

Key observations:
- `gunbc-resolve` is the heaviest internal consumer (9 workspace deps).
- `gunbc-exec` is clean (1 dep: `gunbc-ir`).
- `daglang-lower` contains the evaluator but does not depend on exec,
  transport, or primitives — only on syntax, typecheck, contract, ir.
- `gunbc-test` depends on `gunbc-primitives`. This dependency must be
  removed before primitives can be deleted.

---

## Phase 0 — Eliminate manual registries

**What:** Eliminate all manually maintained callable registries. Two parts:
standalone builtins (small, do first) and pipe methods (larger, same
principle).

**Policy decision (must come first):** No new `extern` declarations.
Operations are either DSL `fn` items (evaluated by the evaluator) or
type-intrinsic (implemented by the evaluator as part of what the type
means, like `+` is part of what `Int` means). There is no third category.
See `src/ARCHITECTURE.md` "Type-intrinsic operations" for rationale.

### 0a — Standalone builtins to DSL fn items

**Add:**

```dag
// dsl/std/logic.dag
fn eq(a: String, b: String) -> Bool {
  a == b
}
```

```dag
// dsl/std/unicode.dag  (already has char_width, char_display_width)
fn code_point(c: Char) -> Int {
  c   // Char is Int where brand("Char") — identity
}
```

```dag
// dsl/gunbc/auth/patterns.dag  (already has the call site)
fn build_token(
  payload: Secret,
  scheme: AuthScheme,
  header_name: String,
  source_id: String,
  required_scopes: List<String>
) -> AccessToken {
  { token: payload, scheme: scheme, expires_at: None }
}
```

`chars` is already a pipe method — the standalone function form is
redundant. Its 2 call sites (`width.dag`, `unicode.dag`) use
`chars(s: text)` syntax. Replace with `text |> chars()`.

**Delete:**

| File | What to delete |
|------|---------------|
| `daglang-typecheck/src/lib.rs` | 4 manual entries in `builtin_callable_contracts()` |
| `daglang-emit/src/fn_codegen.rs` | `"code_point"` and `"chars"` match arms in `compile_call()` (~30 lines) |
| `daglang-lower/src/eval.rs` | `"code_point"` and `"chars"` match arms in `eval_call()` (~25 lines) |

**Verify:**
- `cargo test --workspace --exclude gunbc-dag-tests`
- `cargo clippy --all-targets -- -D warnings`
- Grep: zero standalone entries in `builtin_callable_contracts`.

**Risk:** Low. Each function has 1–2 call sites. The DSL equivalents are
trivial.

### 0b — Pipe methods: DSL fn items + type-intrinsic ops

**Parser change:** Parse `|> name(args)` generically. Remove parse-time
pipe method classification. Produce `Expr::Pipe(expr, name, args)` where
`name` is a string, not a `PipeMethod` enum variant.

**Write DSL fn items** for the ~13 methods expressible via `fold`:
`map`, `filter`, `filter_map`, `flat_map`, `any`, `all`, `contains`,
`count`, `sum`, `first`, `last`, `max_by`, `enumerate`. Place in
`dsl/std/collections.dag`.

**Keep as type-intrinsic** the ~8 operations that need host
implementation: `fold`, `append`, `sort_by` (on List); `chars`, `split`,
`join`, `starts_with`, `ends_with` (on String); `to_bytes`, `to_json`,
`hash` (on Any). The evaluator handles these the same way it handles `+`
— they're part of what the type means.

**Typechecker change:** Resolve pipe method names from AST (DSL `fn`
definitions) and from a built-in type intrinsic table (derived from the
built-in type definitions, not a manual registry). The intrinsic table
is structural: `List` has `fold`, `append`, `sort_by`. `String` has
`chars`, `split`, etc.

**Delete:**
- `PIPE_METHOD_REGISTRY` static array (~260 lines)
- `PipeMethod` enum (~40 lines)
- `builtin_callable_contracts()` pipe method derivation loop
- `emit_family`, `collection_op` classification metadata
- Emit string-name matching in `compile_pipe()` — replace with dispatch
  on resolved method kind (intrinsic vs DSL fn)

**Verify:**
- `cargo test --workspace`
- Grep: zero references to `PIPE_METHOD_REGISTRY` or `PipeMethod`.
- All existing pipe method usage in `.dag` files works unchanged.

**Risk:** Medium. Parser and typechecker changes affect the entire DSL
surface. Phase 0b should be its own PR, separate from 0a.

### 0c — Structured diagnostics (fail-closed)

**Rationale for doing this early:** The postmortem shows that many of
the worst failures came from `eprintln` + continue, unknown-type
fallback, and placeholder rendering. These are exactly the behaviors
that will make a large refactor feel haunted — silent degradation
masking real errors during crate restructuring. Fail-closed must be in
place before the structural changes in Phases 1–7.

**Add to `clippy.toml` (root):**

```toml
[[disallowed-macros]]
path = "std::eprintln"
reason = "Use Diagnostic enum. See src/README.md."
```

**Migrate all `eprintln!("warning: ...")` sites:**

| Site | Current behavior | After migration |
|------|-----------------|-----------------|
| `daglang-lower` ~L8196 | `eprintln` + return false | `Err(LowerError)` |
| `daglang-lower` ~L9922 | `eprintln` + continue | `Err(LowerError)` |
| `daglang-lower` ~L11712 | `eprintln` + continue | `Err(LowerError)` |
| `daglang-emit` type_mapping.rs | `eprintln` + verbatim | `Err(EmitError)` |
| `ir` type_registry.rs | `eprintln` + `ValueBacking::Json` | `Err(TypeError)` |
| `ir` type_shape.rs | `eprintln` + `TypeShape::Opaque` | `Err(TypeError)` |
| `codegen` testgen | `eprintln` + `Json(Null)` | `Err(MockGenError)` |
| `exec` ci_context.rs | `writeln!().ok()` | Document as intentional |

**Also delete:**
- `identity_type_unknown_emits_name_verbatim` test (validates a fallback)
- `AutoTestgenResult::Skipped` variant + placeholder rendering path
- `allow_unresolved_imports` flag in `TypecheckOptions` (once 0a/0b
  make all callables derivable from AST)

**Verify:**
- `cargo clippy --all-targets -- -D warnings` — no `eprintln!` in libs
- `cargo test --workspace` — no test depends on fallback behavior

**Risk:** Medium. Some tests may expect graceful degradation. Those
tests need updating to either provide valid inputs or expect errors.

---

## Phase 1 — Consolidate resolver pure ops into one dispatch function

**What:** Replace the 10 individual pure-value op structs in the resolver
with a single `execute_pure_primitive()` function. This eliminates the
semantic duplication between the resolver and the evaluator without changing
crate boundaries.

**Current state:** 10 separate structs, each implementing `Executable`:

| Struct | Lines (approx) | What it does |
|--------|----------------|-------------|
| `GetFieldOp` | ~30 | `map.get(field_name)` |
| `StringInterpolateOp` | ~25 | Concatenate parts with `value_to_string()` |
| `BinaryOpOp` | ~50 | Dispatch on op kind, call `eval_binop()` |
| `UnaryOpOp` | ~20 | `Not` → `!value_truthy()`, `Neg` → negate |
| `ConditionalOp` | ~20 | `if value_truthy(cond) { then } else { else }` |
| `MatchDispatchOp` | ~30 | Delegate to `eval_match()` |
| `RecordConstructOp` | ~15 | Collect named inputs into `Value::Map` |
| `NullCoalesceOp` | ~15 | `value ?? default` |
| `VariantConstructOp` | ~20 | Build `Value::Map` with `_variant` tag |
| `ListConstructOp` | ~15 | Collect numbered inputs into `Value::List` |

**Replace with:**

```rust
// resolve.rs
fn execute_pure_primitive(
    kind: &PrimitiveOpKind,
    inputs: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    match kind {
        PrimitiveOpKind::GetField => { ... }
        PrimitiveOpKind::StringInterpolate => { ... }
        PrimitiveOpKind::BinaryOp(op) => { ... }
        // etc.
    }
}
```

The match body for each arm is the same logic currently in each struct's
`execute()` method — but now in one function, dispatching on the enum
variant. Where the structs already delegate to `eval.rs` (BinaryOp →
`eval_binop()`, MatchDispatch → `eval_match()`), the new function does the
same. Where the structs have independent implementations, the new function
consolidates them.

A single wrapper struct `PurePrimitiveOp { kind: PrimitiveOpKind }` implements
`Executable` by calling `execute_pure_primitive()`. The resolver instantiates
this instead of 10 different structs.

**Delete:**
- `GetFieldOp`, `StringInterpolateOp`, `BinaryOpOp`, `UnaryOpOp`,
  `ConditionalOp`, `MatchDispatchOp`, `RecordConstructOp`,
  `NullCoalesceOp`, `VariantConstructOp`, `ListConstructOp`
  — 10 struct definitions + 10 `impl Executable` blocks (~240 lines).

**Keep:**
- `FnBodyCallableOp` — has special `__out:` port passthrough logic and data
  values threading. Consolidate later (Phase 4).
- `DeclaredOutputCallableOp` — port name remapping, not pure computation.
- `CallParamSourceOp`, `LiteralSourceOp` — trivial passthrough, could fold
  into the dispatch but low priority.
- `ResourceAcquireOp`, `ResourceReleaseOp`, `DslFsEnvOp` — infrastructure.
- All service ops — Phase 3.

**Verify:**
- `cargo test --workspace --exclude gunbc-dag-tests`
- `cargo test -p gunbc-dag-tests` — generated tests exercise every
  primitive through the resolver. All must pass unchanged.
- Diff the op dispatch function against eval.rs equivalents to confirm
  semantic agreement. Any disagreement is a latent bug being fixed.

**Risk:** Medium. The 10 ops have been tested in production. The new dispatch
function must produce identical outputs for identical inputs. The generated
DAG tests are the verification gate — they exercise every primitive kind.

---

## Phase 2 — Migrate eval.rs functions into the dispatch

**What:** Where Phase 1 kept independent implementations in the dispatch
function, this phase replaces them with calls to `eval.rs` functions. After
this, the dispatch function is a thin adapter between port-based inputs and
the evaluator's API.

The adapter's job: extract values from the `inputs` HashMap by port name,
call the corresponding `eval.rs` function, and package the result into an
`outputs` HashMap.

**Specifically:**

| Dispatch arm | Replace with |
|---|---|
| `GetField` | `eval_field_access(object, field_name)` (new pub fn in eval.rs) |
| `StringInterpolate` | `eval_string_interpolation(parts)` (new pub fn) |
| `BinaryOp` | `eval_binop(lhs, op, rhs)` (already public) |
| `UnaryOp` | `eval_unary(op, val)` (new pub fn) |
| `Conditional` | `value_truthy(cond)` + select (already public) |
| `MatchDispatch` | `eval_match(scrutinee, arms, ...)` (already public) |
| `RecordConstruct` | `eval_record(fields)` (new pub fn) |
| `NullCoalesce` | `eval_null_coalesce(a, b)` (new pub fn) |
| `VariantConstruct` | `eval_variant(tag, fields)` (new pub fn) |
| `ListConstruct` | `eval_list(items)` (new pub fn) |

Some of these are already public in eval.rs. Others need small public
wrappers around internal logic. Each new wrapper is ≤5 lines.

**Delete:** The independent implementations in the dispatch function from
Phase 1 (~150 lines). Replaced by evaluator calls.

**Verify:**
- Same test gates as Phase 1.
- After this phase, every pure primitive's semantics are defined exactly
  once: in `eval.rs`. The dispatch function is purely mechanical (port name
  extraction + evaluator call + output packaging).

**Risk:** Low, given Phase 1 already verified the dispatch structure.

### Also in Phase 2: Embed data values in the DAG

Eliminate the `data_values` sidecar. During lowering, `data` declarations
should produce literal nodes in the DAG (the lowerer already does this for
service call arguments via `CallLiteralSource`). Fn bodies that reference
data declarations get input ports wired to those literal nodes.

**Delete:**
- `data_values: HashMap<String, serde_json::Value>` from `LowerOutput`,
  `CompileOutput`, `CompileLoweredResult`, `CachedCompileData`
- `evaluate_fn_body_with_data()` — replace with `evaluate_fn_body()`.
  Data values arrive through normal input ports, not a sidecar.
- `#[serde(default)]` cache compat hack on `data_values`
- `json_to_eval_value()` conversion function (no longer needed at the
  eval boundary — values arrive as `Value`, not `serde_json::Value`)

**Verify:**
- `cargo test --workspace`
- Grep: zero references to `data_values` as a HashMap sidecar.
- Data declaration tests still pass (values now flow through edges).

**Risk:** Medium. The lowerer must wire data declaration literal nodes to
fn body input ports, which requires extending `wire_fn_call_arguments()`
to handle data idents in fn body contexts the same way it handles them in
service call contexts.

---

## Phase 3 — Consolidate service prepare/parse ops

**What:** The 4 prepare ops and 4 parse ops are structurally identical —
they differ only in which transport kind (REST, Shell, File, Local) they
target. Each reads a spec and mechanically formats inputs into a request or
extracts outputs from a response.

Replace with 2 parameterized ops:

```rust
struct GenericPrepareOp {
    transport_kind: TransportKind,  // Rest | Shell | File | Local
    spec: OperationSpec,
}

struct GenericParseOp {
    transport_kind: TransportKind,
    spec: OperationSpec,
}
```

The `execute()` method dispatches on `transport_kind` for the few places
where behavior differs (URL interpolation for REST, argv construction for
Shell, etc.).

**Delete:**
- `GenericRestPrepareOp`, `GenericShellPrepareOp`, `GenericFilePrepareOp`,
  `GenericLocalPrepareOp` — 4 structs (~400 lines total in
  `service_ops_impl.rs`)
- `GenericRestParseOp`, `GenericShellParseOp`, `GenericFileParseOp`,
  `GenericLocalParseOp` — 4 structs (~350 lines total)
- Replaced by 2 parameterized structs (~300 lines total)

**Also consolidate:**
- `InterfaceStubPrepareOp`, `InterfaceStubExecuteOp`,
  `InterfaceStubParseOp` — 3 structs into 1 `InterfaceStubOp` with
  internal phase dispatch.

**Keep:**
- `FilesystemExecuteOp` — direct I/O, not transport-mediated. Stays.

**Verify:**
- `cargo test --workspace`
- Service op tests in `gunbc-resolve` and generated tests that exercise
  transport triplets.

**Risk:** Medium. Service ops have protocol-specific edge cases (REST auth
headers, Shell env vars, File path normalization). Parameterization must
preserve all edge cases. Review each transport kind's `execute()` body
carefully during consolidation.

---

## Phase 4 — Collapse FnBodyCallableOp into the dispatch

**What:** `FnBodyCallableOp` is currently separate from the pure primitive
dispatch because it has extra concerns:
- `__out:` port passthrough logic (forwarding ports that bypass fn evaluation)
- Optional `fn_body` (some callables have no body — they're just port maps)

After Phases 1–2, the pure dispatch function already handles the common
case, and `data_values` has been embedded in the DAG (Phase 2). Extend
the dispatch to handle `LoweredOp::Callable` with `fn_body: Some(...)`:

```rust
fn execute_lowered_op(
    op: &LoweredOp,
    inputs: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    match op {
        LoweredOp::Primitive(kind) => execute_pure_primitive(kind, inputs),
        LoweredOp::Callable { fn_body: Some(body), .. } => {
            let mut outputs = evaluate_fn_body(body, inputs, ...)?;
            // handle __out: passthrough ports
            for (k, v) in inputs {
                if k.starts_with("__out:") { outputs.insert(k.clone(), v.clone()); }
            }
            Ok(outputs)
        }
        LoweredOp::Callable { fn_body: None, .. } => {
            // DeclaredOutputCallableOp: port forwarding
            ...
        }
        ...
    }
}
```

**Delete:**
- `FnBodyCallableOp` struct + `impl Executable` (~60 lines)
- `DeclaredOutputCallableOp` struct + `impl Executable` (~40 lines)

**What remains in the resolver after this phase:**
- `execute_lowered_op()` — single dispatch function for all pure ops + callables
- `PurePrimitiveOp` wrapper struct (implements `Executable`, calls dispatch)
- `GenericPrepareOp`, `GenericParseOp` — transport formatting (Phase 3)
- `InterfaceStubOp` — interface capability stubs
- `ResourceAcquireOp`, `ResourceReleaseOp`, `DslFsEnvOp` — infrastructure
- `FilesystemExecuteOp` — direct file I/O
- `CallParamSourceOp`, `LiteralSourceOp` — trivial passthrough

The resolver is now a thin translation layer: `LoweredOp` → one of ~6 op
types (down from ~30).

**Verify:**
- `cargo test --workspace`
- Specifically verify `gist.dag` multi-entrypoint scenarios (the shared fn
  node fix from this postmortem) — fn body evaluation must still work
  correctly per-entrypoint.

**Risk:** Medium. The `__out:` passthrough logic is subtle. Verify with
generated tests that exercise fn bodies with data declarations (now
arriving as normal input ports after Phase 2's data embedding).

---

## Phase 5 — Delete `gunbc-primitives`

**What:** After Phases 1–4, the evaluator handles all pure operations. The
`PrimitiveOp` enum in `gunbc-primitives` is a parallel definition of the
same operations, used by the old code path.

Audit remaining consumers:
- `gunbc-resolve` — uses `PrimitiveOp` variants during resolution. After
  Phases 1–4, the resolver dispatches on `PrimitiveOpKind` (from
  `daglang-lower`) instead. Remove `gunbc-primitives` dependency.
- `gunbc-test` — uses `PrimitiveOp` for mock construction and transport
  pattern matching. Must be migrated to use `LoweredOp` or transport-level
  abstractions from `gunbc-ir`.
- `gunbc-codegen` — uses `PrimitiveOp` for test generation. Must be
  migrated similarly.

**Migrate `gunbc-test`:**
The test crate uses `PrimitiveOp` in `Mockable` trait implementations and
`MockOp` construction. The migration path:
- `MockOp` and `Mockable` should dispatch on `LoweredOp` variants or on a
  trait bound, not on `PrimitiveOp` enum variants.
- Transport-related test concerns (dry-run interception, mock transport
  responses) already use `TransportOps` from `gunbc-lib-transport`, not
  `PrimitiveOp`.

**Delete:**
- `src/08_materialize/primitives/` — entire crate
- `gunbc-primitives` from workspace `Cargo.toml`
- All `use gunbc_primitives::` statements (resolve, test, codegen)

**Keep (relocated):**
- I/O prepare op logic (PrepareFileRead, PrepareFileWrite, PrepareShell) —
  these are transport request builders. Move to `gunbc-lib-transport` where
  they conceptually belong (building requests for the transport layer).

**Verify:**
- `cargo test --workspace`
- Confirm no remaining references to `gunbc_primitives` in any crate.

**Risk:** High. `gunbc-test` is load-bearing for the test infrastructure.
The `Mockable` trait migration must be done carefully. Plan a sub-phase
where `gunbc-test` is updated first, then `gunbc-primitives` is deleted.

**Sub-phasing:**
- 5a: Remove `gunbc-primitives` dependency from `gunbc-resolve` (should be
  straightforward after Phases 1–4).
- 5b: Migrate `gunbc-test` from `PrimitiveOp` to `LoweredOp`-based
  dispatch. Add `daglang-lower` as a dependency of `gunbc-test` (or move
  the needed types to `gunbc-ir`).
- 5c: Migrate `gunbc-codegen` similarly.
- 5d: Delete `gunbc-primitives` crate.

---

## Phase 6 — Extract the evaluator

**What:** After Phases 1–5, `eval.rs` in `daglang-lower` is the single
implementation of all pure operation semantics. It is consumed by:
- `daglang-lower` itself (compile-time: `build_data_values()`)
- `gunbc-resolve` (runtime: dispatch function from Phase 4)
- `gunbc-codegen` (compile-time: `fidelity.rs` stdlib evaluation)

The evaluator's own dependencies are minimal: `gunbc-ir` (for `Value`,
node types) and `daglang-syntax` (for `PipeMethod`, `BinOp` enums used in
`LoweredExpr` variants).

**Problem:** `LoweredExpr`, `LoweredFnBody`, `LoweredBinOp`,
`LoweredMatchArm` are defined in `daglang-lower::expr`. The evaluator
interprets these types. Extracting the evaluator requires that these types
be accessible from outside `daglang-lower`.

**Option A — Move expression types to `gunbc-ir`:**
These are IR types (intermediate representation of expressions). They
belong in the IR crate. However, they reference `PipeMethod` and `BinOp`
from `daglang-syntax`, which would make `gunbc-ir` depend on
`daglang-syntax`. Currently `gunbc-ir` has no dependency on syntax. This
is a significant change.

**Option B — Create `daglang-expr` leaf crate:**
A small crate (~500 lines) containing just the expression types. Both
`daglang-lower` and the new evaluator crate depend on it. No new dependency
for `gunbc-ir`.

**Option C — Re-derive expression enums from `gunbc-ir` primitives:**
Define `BinOp`, `UnaryOp`, etc. in `gunbc-ir` independently of
`daglang-syntax`. The lowerer maps from syntax-level enums to IR-level
enums during lowering. The evaluator works with IR-level enums only. This
duplicates the enum variants but removes the cross-layer dependency.

**Recommended: Option B.** Smallest blast radius, no new deps on existing
crates, clean separation.

**New crate: `daglang-expr`** (in `src/05_graph/daglang-expr/`):

Dependencies: `gunbc-ir`, `daglang-syntax` (for PipeMethod, BinOp reuse).

Contents (moved from `daglang-lower/src/expr.rs`):
- `LoweredExpr` enum
- `LoweredFnBody` struct
- `LoweredBinOp`, `LoweredUnaryOp`
- `LoweredPattern`, `LoweredMatchArm`
- `LoweredLiteral`
- `LoweredStringPart`

**New crate: `daglang-eval`** (in `src/05_graph/daglang-eval/`):

Dependencies: `gunbc-ir`, `daglang-expr`.

Contents (moved from `daglang-lower/src/eval.rs`):
- `evaluate_fn_body()`, `evaluate_fn_body_with_data()`
- `eval_expr()`, `eval_binop()`, `eval_match()`, `eval_pipe_method()`
- `value_truthy()`, `value_to_string()`, `sort_key()`
- `EvalError`
- Internal `Env` struct

**Update consumers:**
- `daglang-lower`: depends on `daglang-expr` + `daglang-eval`. Uses
  `daglang-eval::evaluate_fn_body()` in `build_data_values()`. Constructs
  `LoweredExpr` from `daglang-expr` types.
- `gunbc-resolve`: depends on `daglang-eval` instead of `daglang-lower`
  for evaluation. **Drops `daglang-lower` dependency entirely** if no other
  lowerer exports are used (verify).
- `gunbc-codegen` (`fidelity.rs`): depends on `daglang-eval` instead of
  `daglang-lower` for `evaluate_fn_body()`.

**Delete:**
- `daglang-lower/src/eval.rs` — moved to `daglang-eval`.
- `daglang-lower/src/expr.rs` — moved to `daglang-expr`. `daglang-lower`
  re-exports from `daglang-expr` for backwards compat during transition,
  then remove re-exports once all consumers are updated.

**Resulting dependency graph (relevant subset):**

```
daglang-syntax          (leaf)
gunbc-ir                (infra, contract, delegate-macros)
    ↓
daglang-expr            (ir, syntax)    ← NEW
    ↓
daglang-eval            (ir, expr)      ← NEW
    ↓
daglang-lower           (contract, syntax, typecheck, ir, expr, eval)
    ↓
gunbc-resolve           (eval, ir, transport, blob)  ← SLIMMED
    ↓
gunbc-exec              (ir)            ← UNCHANGED
```

Key improvement: `gunbc-resolve` no longer depends on `daglang-lower`,
`daglang-driver`, `daglang-derive`, or `gunbc-primitives`. Its dependency
set shrinks from 9 workspace deps to ~5.

**Verify:**
- `cargo test --workspace`
- Confirm `gunbc-resolve` no longer depends on `daglang-lower`.
- Confirm `daglang-lower` re-export removal doesn't break any consumer.

**Risk:** Medium. Type moves across crate boundaries require updating every
import path. Use re-exports during transition to avoid a single massive
commit.

---

## Phase 7 — Create `gunbc-interp` (the interpreter crate)

**What:** After Phase 6, the resolver is a thin layer that wraps
`execute_lowered_op()` (a single dispatch function) in `Executable` trait
impls, plus service op transport wiring. This is the interpreter — make it
explicit.

**New crate: `gunbc-interp`** (in `src/08_materialize/interp/`):

This crate IS the interpreter. Its job: take `VerifiedDag<LoweredOp>` and
run it. It combines:

- The `execute_lowered_op()` dispatch function (from Phases 1–4, currently
  in `gunbc-resolve`)
- Service op transport wiring (GenericPrepareOp, GenericParseOp,
  InterfaceStubOp, FilesystemExecuteOp — from Phase 3, currently in
  `gunbc-resolve/service_ops/`)
- `dry_run.rs` (mock wiring for dry-run execution)

**Explicitly NOT in `gunbc-interp`:**
- `builder.rs` (DSL graph compilation + dagbin cache) — this is
  orchestration, not interpretation. Stays in `daglang-driver` or a thin
  runtime orchestration layer above the interpreter. The interpreter's
  contract is "consume `VerifiedDag<LoweredOp>` and run it," not "compile
  and run."
- `fs_env.rs` (filesystem environment setup) — orchestration concern.
  The interpreter receives environment configuration, it doesn't discover
  it.

Dependencies:
- `gunbc-ir` — DAG/Node/Port/Edge/Value types
- `daglang-eval` — pure operation evaluation
- `daglang-expr` — expression IR types (transitive via eval)
- `gunbc-lib-transport` — I/O execution
- `gunbc-lib-blob` — blob content acquisition

Public API:

```rust
/// Run a compiled DAG directly. No resolution step.
pub fn interpret(
    dag: &VerifiedDag<LoweredOp>,
    config: InterpretConfig,
) -> Result<InterpretResult, InterpretError>
```

The interpreter dispatches each node to one of:
- **Pure operation** → `daglang-eval` (field access, binary ops, fn bodies,
  collection ops, pattern matching, conditionals, etc.)
- **Transport prepare/parse** → service op wiring (spec-driven request/
  response formatting)
- **Transport execute** → `gunbc-lib-transport` (shell, HTTP, filesystem)
- **Infrastructure** → resource acquire/release, fs env, param source,
  literal source

**What this enables:**
- `VerifiedDag<LoweredOp>` is directly interpretable. No resolution step.
- The full path becomes: parse → resolve imports → typecheck → lower →
  interpret. No "materialize" phase.
- The interpreter and compiler are structurally separate — they share
  `gunbc-ir` and `daglang-eval`, nothing else.

**Delete:**
- `gunbc-resolve` — entire crate. All content either moved to
  `gunbc-interp` (service ops, builder, fs_env, dry_run) or deleted
  (resolve.rs op wrappers, already emptied by Phases 1–4).
- `gunbc-exec` scheduling logic absorbed into `gunbc-interp`. If
  `gunbc-exec` still has value as the generic `Executable` trait + `DynOp`
  type for test mocks, keep it as a slim trait crate. Otherwise fold into
  `gunbc-ir`.

**`gunbc-exec` disposition:**
`gunbc-exec` currently provides:
- `Executable` trait
- `execute_dag()` function (topological scheduler)
- `ExecutionMode` (DryRun/Live)
- `BoundaryMock` / interception
- Progress tracking, display, CI context

Options:
- **Keep `gunbc-exec` as the generic scheduler.** `gunbc-interp` depends on
  it and provides the `LoweredOp`-specific dispatch. `gunbc-exec` remains
  type-generic — it schedules any `Dag<T: Executable>`.
- **Fold scheduler into `gunbc-interp`.** If nothing else needs the generic
  scheduler, it's unnecessary abstraction.

Recommended: **Keep `gunbc-exec` as generic scheduler.** It's already clean
(1 dep), and the test infrastructure uses `DynOp` + `MockOp` through it.
`gunbc-interp` implements `Executable for LoweredOp` and calls
`gunbc-exec::execute_dag()`.

**Resulting dependency graph:**

```
COMPILER:                          INTERPRETER:
  daglang-syntax                     daglang-eval (ir, expr)
  daglang-resolve                         ↓
  daglang-typecheck                  gunbc-interp (ir, eval, exec, transport, blob)
  daglang-expr (ir, syntax)               ↓
  daglang-lower (ir, expr, eval)     gunbc-exec (ir)  ← generic scheduler
  daglang-derive (ir, lower)         gunbc-lib-transport (ir, exec)
  daglang-emit (ir, lower, ...)
  daglang-driver (orchestrator)
```

The interpreter depends on 5 workspace crates (ir, eval, exec, transport,
blob). The resolver had 9. The interpreter does not depend on any compiler
crate (syntax, typecheck, lower, derive, emit, driver).

**Verify:**
- `cargo test --workspace`
- Confirm `gunbc-interp` has no dependency on any `daglang-*` compiler
  crate except `daglang-eval` and `daglang-expr`.
- Confirm all existing integration tests pass through the new interpreter
  path.
- Benchmark: DAG execution should be slightly faster (one fewer indirection
  layer, no type-erasure overhead for pure ops).

**Risk:** High. This is the structural culmination of Phases 1–6. All
prior phases must be stable. The migration path: implement `gunbc-interp`
alongside `gunbc-resolve`, migrate consumers one at a time, delete
`gunbc-resolve` once no consumers remain.

---

## Phase 8 — Clean up scaffolding and residual debt

**What:** Delete temporary scaffolding identified in the postmortem that
should now be unnecessary.

| Scaffolding | Removal condition | Phase that enables it |
|---|---|---|
| Synthesized `expr_value_*` fallback nodes in lowerer | Structural lowering handles complex pure args | Phase 2 (evaluator handles all pure ops) |
| `AutoTestgenResult::Skipped` + placeholder rendering | Fail-closed testgen | Phase 0c (structured diagnostics) |
| `gunbc-tests` glob-include `src/generated/*.rs` scaffold | Stable testgen output model | Phase 5 (clean test infrastructure) |
| Manual dagbin cache epoch bumps | Compiler version in cache key | Phase 7 (evaluator-based execution) |
| `allow_unresolved_imports` flag in TypecheckOptions | All callables derived from AST | Phase 0a/0b (builtins + pipe methods → DSL) |

**Verify:**
- `cargo test --workspace`
- `make test-all` (if applicable after generated test restructuring)

---

## Summary: what gets deleted, moved, and created

| Phase | What happens | Crates affected |
|-------|-------------|-----------------|
| 0a | Delete ~55 lines Rust (builtins), add ~10 lines DSL | typecheck, emit, lower |
| 0b | Delete ~300 lines (pipe method registry → DSL fn + intrinsics) | syntax, typecheck, emit, lower, eval |
| 0c | Delete ~100 lines (fallback paths → hard errors, `eprintln` ban) | lower, emit, ir, codegen |
| 1 | Delete ~240 lines (10 op structs → 1 dispatch fn) | resolve |
| 2 | Delete ~150 lines (independent impls → eval.rs calls) + embed data values | resolve, lower (+~30) |
| 3 | Delete ~450 lines (8 service structs → 2 parameterized) | resolve/service_ops |
| 4 | Delete ~100 lines (FnBodyCallableOp → dispatch) | resolve |
| 5 | Delete ~800 lines (entire crate) | primitives (deleted), test, codegen |
| 6 | Move ~1,300 lines (eval.rs + expr.rs → new crates) | lower, new: expr + eval |
| 7 | Delete ~400 lines (resolve → interp), create gunbc-interp | resolve (deleted), new: interp |
| 8 | Delete ~50 lines (scaffolding) | lower, codegen, tests |

**Net result:**

| | Before | After |
|---|---|---|
| Crates hosting interpreter logic | 3 (lower, resolve, primitives) | 1 (interp) |
| Implementations of "map a list" | 3 (eval.rs, resolver, primitives) | 1 (eval) |
| Resolver workspace deps | 9 | N/A (deleted) |
| Interpreter workspace deps | N/A | 5 (ir, eval, exec, transport, blob) |
| Manual builtin entries | 4 | 0 |
| Manual pipe method registry entries | 21 | 0 (8 type-intrinsic + 13 DSL fn) |
| `eprintln` fallback sites | ~8 | 0 |
| `data_values` sidecar handoff points | ~6 | 0 (embedded in DAG) |

**Crates deleted:** `gunbc-primitives` (Phase 5), `gunbc-resolve` (Phase 7).

**Crates created (from moved code, not new logic):**
- `daglang-expr` (~500 lines, expression IR types from lower) — Phase 6
- `daglang-eval` (~800 lines, evaluator from lower) — Phase 6
- `gunbc-interp` (~300 lines new + ~500 moved from resolve) — Phase 7

---

## Sequencing and dependencies between phases

```
Phase 0a (builtins → DSL)          — independent, do first
Phase 0b (pipe methods → DSL/intrinsic) — independent, do early
Phase 0c (structured diagnostics)  — independent, do before structural changes
    ↓
Phase 1 (consolidate pure ops)     — after 0c (fail-closed in place)
    ↓
Phase 2 (evaluator delegation + data embed) — depends on Phase 1
    ↓
Phase 3 (service op consolidation) — independent of 1–2, can parallel
    ↓
Phase 4 (FnBodyCallableOp)         — depends on Phase 2
    ↓
Phase 5 (delete primitives)        — depends on Phase 4
    ↓
Phase 6 (extract evaluator)        — depends on Phase 5
    ↓
Phase 7 (create gunbc-interp)      — depends on Phase 6
    ↓
Phase 8 (scaffolding cleanup)      — depends on Phase 7
```

Phases 0a, 0b, 0c, and 3 can proceed in parallel. The critical path is
0c → 1 → 2 → 4 → 5 → 6 → 7 → 8.

---

## Invariant: "Can you describe what this stage does in one sentence?"

Every stage in the pipeline should pass this test. If you can't describe
what a crate does without saying "and also," it's doing too much.

**After this refactor:**

| Crate | One sentence |
|-------|-------------|
| `daglang-syntax` | Parses `.dag` source into an AST. |
| `daglang-resolve` | Discovers `.dag` files and builds the import graph. |
| `daglang-typecheck` | Validates types and produces a typed project. |
| `daglang-expr` | Defines the expression IR types (`LoweredExpr`, `LoweredFnBody`). |
| `daglang-eval` | Evaluates expression IR to produce values. |
| `daglang-lower` | Lowers typed AST to graph IR (`VerifiedDag<LoweredOp>`). |
| `daglang-derive` | Extracts metadata (manifest, obligations) from graph IR. |
| `daglang-emit` | Generates target-language code from graph IR. |
| `daglang-driver` | Orchestrates the compiler pipeline. |
| `gunbc-interp` | Interprets graph IR: pure ops via eval, I/O via transport. |
| `gunbc-exec` | Schedules DAG nodes in topological order. |
| `gunbc-lib-transport` | Executes shell commands, HTTP requests, and file I/O. |

**Stages that fail this test today:**

| Crate | Problem |
|-------|---------|
| `daglang-lower` | Lowers typed AST to graph IR **and also** contains the runtime expression evaluator **and also** evaluates compile-time data declarations. |
| `gunbc-resolve` | Resolves `LoweredOp` to `DynOp` **and also** reimplements pure operations **and also** wires transport specs **and also** builds DSL graphs **and also** manages the dagbin cache. |
| `gunbc-primitives` | **DELETED** (2026-03-09). Moved `filename.rs` to `gunbc-ir`, inlined `FsEnv` to `gunbc-resolve`. |

---

## Remaining fail-closed gaps (updated 2026-03-10)

Tracked here for the next cleanup pass. Each item is a place where the
compiler silently fabricates values or accepts degraded input instead of
failing with a structured diagnostic.

All 6 items share one root pattern: **something fails → the stage
fabricates `Unknown`/`Json`/`identity(...)`/verbatim-name/empty-output →
the compiler keeps moving.** Fixing them means every stage either
succeeds with correct data or fails with a structured diagnostic.

### FC-1. Callable return wiring silently discarded

**Severity:** P1 — direct postmortem invariant violation ("fail closed")

`daglang-lower/src/lib.rs:8137` does
`let _ = wire_callable_return_outputs(...)` for non-fn callable items.
The comment explicitly says: "TODO: make this a hard error once the
lowerer can trace through service call result bindings."

The other call site (line 4200, loop body construction) correctly checks
`.is_err()` and returns `None`, proving the pattern is solvable.

The failure is legitimate today — service call result bindings can't be
traced as simple idents — but the silent discard masks real wiring bugs.
Service call arg wiring (`wire_service_call_argument`) already uses
`LowerError::WiringFailure` to fail hard, so this is an inconsistency
within the same file.

**Prerequisite:** Lowerer must trace through service call result
bindings (let-bound service call outputs referenced in return
expressions).

**Fix:** Once the prerequisite is met, convert `let _` to `?` and
propagate `LowerError::WiringFailure`.

### FC-2. Type system / emit fallbacks

**Severity:** P2 — degraded output rather than hard failure

Five sites turn structured errors back into defaults:

| # | Site | File:Line | Behavior |
|---|------|-----------|----------|
| a | `emit_type()` | `daglang-emit/src/type_mapping.rs:46` | `type_shape()` failure → `Opaque("Unknown")` |
| b | `emit_platform_type()` | `daglang-emit/src/type_mapping.rs:153` | No scalar match → `model.opaque_fallback` |
| c | `resolve_and_emit()` | `daglang-emit/src/type_mapping.rs:180` | Unresolved type → raw name verbatim |
| d | `value_backing_for_type_id()` | `ir/src/types.rs:1129` | Unknown type → `.unwrap_or(ValueBacking::Json)` |
| e | `register_type_def()` | `daglang-typecheck/src/lib.rs:1001,1191,1274` | Unresolved alias/field → `identity(...)` placeholder |

Additionally, `auto_mock.rs:226,680` uses `unwrap_or(ValueBacking::Json)`
for mock value generation, propagating the same default downstream.

**Fix direction:** Convert each to `Result` propagation, working from
the most downstream site (d: `value_backing_for_type_id`) upward to (a:
`emit_type`). Each conversion makes the next one easier because callers
are already forced to handle errors.

### FC-3. `TypeRegistry::is_compatible()` too permissive

**Severity:** P2 — hides real inference / lowering mistakes

`ir/src/type_registry.rs:877-880`:

```rust
// Source Any/Unknown: inferred types that couldn't be resolved
// (e.g. fold's generic return). Compatible with any target until
// the type system supports generics.
if from.0 == "Any" || from.0 == "Unknown" {
    return true;
}
```

And lines 888-890: when `type_shape()` fails for either type,
compatibility falls back to `to == "Json"` or `base_ok`. These are
escape hatches added to keep the compiler moving, but they can hide real
type mismatches.

**Impact:** A lowering bug that produces `Unknown` instead of a concrete
type won't be caught by compatibility checks — it will silently pass
through and surface as a confusing runtime error.

**Dependency:** Should be tightened *after* FC-2 is resolved, because
fixing type fallbacks will surface the real types that `Any`/`Unknown`
was hiding. Then an audit can determine which callers genuinely need the
escape hatch (e.g., `fold`'s generic return) vs. which were masking bugs.

### FC-4. `Map<K,V>` type/runtime incoherence

**Severity:** P2 — invariant violation across stages

The type system accepts `Map<K,V>` with non-String keys:

```rust
// daglang-typecheck/src/lib.rs:1238-1241
// Note: non-String key types are accepted structurally but
// runtime Value::Map is string-keyed. Type checking does not
// reject this — it will surface as a runtime mismatch.
```

But the runtime representation is hardcoded to string keys:

```rust
// ir/src/value.rs:163
Map(BTreeMap<String, Value>),
```

And `value_compatible_with_type_id()` in `ir/src/types.rs:1175-1185`
explicitly rejects non-String keys at runtime validation time. So the
type system says yes, the runtime says no, and the mismatch surfaces
late as a confusing runtime error.

**Fix (simplest):** Reject non-String map keys in typecheck. Add a
single guard in `resolve_field_type_dag` that emits a diagnostic when
the key type of `Map<K,V>` is not `String`. Zero downstream impact —
no current DSL module uses non-String map keys.

**Fix (complete, future):** Make `Value::Map` generic over key types.
This is a large change touching every `BTreeMap<String, Value>` site.
Not warranted until there's a real use case for non-String keys.

### FC-5. `gunbc-interp` placeholder behavior, not production-wired

**Severity:** P2 — half-maintained crate with passthrough escape hatch

`gunbc-interp` (created 2026-03-09, 202 lines) is architecturally
correct: 5 deps (ir, eval, lower, exec, transport), no compiler deps,
clean `execute_lowered_op()` dispatch on all `LoweredOp` variants.

But it still has `_ => Ok(inputs)` passthrough at line 141-143 for
all transport/I/O `PrimitiveOpKind` variants. And `gunbc-resolve`
still owns the entire production materialization path: `builder.rs`,
`fs_env.rs`, `dry_run.rs`, `service_ops`, `resolve_lowered_dag_with()`.

So the boundary is cleaner, but not authoritative. The postmortem's
own crate table says `gunbc-interp` "interprets graph IR" but it
doesn't actually own interpretation in production.

**Decision needed:** Either:
- (a) Wire `gunbc-interp` as the production interpreter, moving
  transport dispatch from `gunbc-resolve` service_ops into it, or
- (b) Defer and mark it explicitly as "test/prototype only" so it
  doesn't accumulate half-maintained divergence.

Option (a) benefits from FC-1 through FC-4 being resolved first.

### FC-6. Duplicated builtin/intrinsic registries

**Severity:** P3 — maintenance hazard, not a correctness bug today

Two independent lists define builtin operations:

| Registry | Location | Entries | Purpose |
|----------|----------|---------|---------|
| `builtin_callable_contracts()` | `daglang-typecheck/src/lib.rs:2109-2250` | 30 | Typecheck contract validation |
| `INTRINSIC_CALLS` | `daglang-eval/src/eval.rs:182-187` | 22 | Runtime dispatch markers |

The 8-entry delta (typecheck has `max_by`, `replace_section`, `to_bytes`,
`to_json`, `hash`, and render helpers that eval lacks) is intentional —
those operations are implemented elsewhere. But there is no compile-time
check that the two lists stay in sync, and adding a new intrinsic
requires updating both.

Deleting `|>` and `PIPE_METHOD_REGISTRY` (2026-03-09) removed one
duplicated registry, but this pair remains.

**Fix:** Extract a shared `BuiltinDef` enum or const array in a common
location (e.g., `daglang-eval`, since typecheck already depends on it
transitively). Both crates consume the single source of truth. Each
entry specifies arity, parameter names, and whether it has an eval
implementation — typecheck builds `CallableContract` from it, eval
builds its dispatch table from it.

### Testgen fallback workarounds (related to FC-2)

These are downstream consequences of the type/emit fallbacks above,
tracked separately because they're in testgen rather than the compiler:

| Site | File:Line | Behavior |
|------|-----------|----------|
| Soft lower | `codegen/src/testgen/codegen.rs:4826` | `gunbc_exec::lower(self.dag).ok()` — discards lowering failures, uses unlowered analysis |
| Port type default | `codegen/src/testgen/codegen.rs:4916` | Missing port type → `("String", Cardinality::ONE)` |
| Mock default | `codegen/src/testgen/codegen.rs:7309,7404` | Unknown type / unmatched type → `Json(Null)` |
| Skip modules | `codegen/src/bin/testgen_cli.rs:22` | `TESTGEN_SKIP_MODULES: &[&str] = &["std.patterns", "gunbc.auth.patterns"]` |

These are "quietly keep going" paths that prevent testgen from surfacing
real gaps. They should be converted to structured diagnostics as FC-2 is
resolved, since many of the unknown types will become properly resolved.

### 5. Generated binary `#![allow(clippy::disallowed_macros)]`

Generated CLI binaries need `eprintln!` for user-facing error output.
The codegen renderer adds `#![allow(clippy::disallowed_macros)]` to
generated `main.rs` files. This is a workaround — ideally generated
binaries would use a structured error reporting path instead of
direct stderr writes.

### 6. Pipe operator deleted (2026-03-09)

`|>` syntax, `Expr::Pipe`, `Expr::PipeCall`, `PipeMethod` enum,
`PIPE_METHOD_REGISTRY`, `EmitCollectionFamily`, and all related
infrastructure deleted. 52 `.dag` usages rewritten to imperative
let+call style. `for` loops kept (they're loop executor constructs
for service call fan-out, not map sugar).

### 7. Mock type generation — fabrication audit (2026-03-10)

After deleting `|>` and converting to `call()` syntax, 529 of 2186
generated tests failed. All 529 (and the remaining 115 after fixes)
are instances of the **same fabrication pattern** described in the
FC taxonomy above: a construction function encounters input it can't
handle and fabricates something plausible instead of failing.

#### What was fixed (529 → 115)

Each fix below is a local patch that compensates for an upstream
fabrication. The patches are correct but they treat symptoms. The
systematic fix is the FC Phase A→D pass that eliminates the upstream
fabrications entirely.

| Fix | Compensates for | Upstream fabrication (FC ref) |
|-----|----------------|------------------------------|
| Use merged registry in `default_value_for_type` | Core-only OnceLock registry | FC-2d: `value_backing_for_type_id` → `Json` |
| `product_witness` container re-wrapping | `base_type()` strips wrappers | FC-2e: `identity(...)` placeholder for unresolved types |
| `eval_call` dotted-name passthrough | Evaluator can't execute service calls | FC-5: interp not production-wired |
| `Env.data_values` propagation | Sibling fns lose data declarations | Missing threading, not a fabrication |
| `collect_local_let_bindings` Node branch | Intrinsic calls excluded from bindings | Inconsistency, not a fabrication |
| Filter placeholder witnesses in codegen | `scalar_witness_for_base` → `Str("<Type>")` | FC-2e: fabricated placeholder |
| `CallableOp` unbound variable fallback | Missing optional input → hard error | Testgen fabrication (skip module gap) |
| `variant_witnesses` → `Value::Enum` | `Value::Str("Variant")` stale convention | Representation inconsistency |
| Inline record parser fix | `parse_type_expr` → `Named("Record")` | FC-2e: parser discards structure |
| `get_by_name` before `resolve_type` | Type expr parser rejects valid names | **Invariant violation** (see below) |
| `ResourceHandle` → `std/resources.dag` | Rust-only type, no DSL definition | Types should be .dag (principle) |
| 39 redundant Rust type registrations deleted | Dual source of truth | Types should be .dag (principle) |
| 13 container aliases → parametric forms | `StringList` vs `List<String>` | Naming inconsistency |

#### Two invariant violations discovered

**IV-1: `register()` accepts names that `resolve_type()` rejects.**
The type registry has two lookup paths: `get_by_name()` (direct HashMap
lookup, always works) and `resolve_type()` (parses the name as a type
expression first). The expression parser rejects names containing commas
or braces — exactly the names inline record types produce (e.g.,
`{key: String, value: String}`). This means you can register a type but
not resolve it. Every callsite of `resolve_type` silently fails for these
types and falls back to fabrication.

**IV-2: Branded types hide their structural base.**
`Char = Int where brand("Char")` creates a type DAG where `base_type()`
returns `"Char"` (the brand name) instead of `"Int"` (the structural
base). Since brand is a compile-time annotation — `Char` IS `Int` at
runtime — any function asking "what's the backing type?" gets the wrong
answer. `scalar_witness_for_base` produces `Str("<Char>")` instead of
`Int(42)`.

Both are instances of the FC pattern: a construction function encounters
input it can't handle (unresolvable name / unknown base type) and
fabricates something plausible (`None` / `Str("<Char>")`).

#### Remaining 115 failures — catalogue

All 115 are downstream consequences of IV-1, IV-2, and FC-2.

**Group A: Char type backing (39 tests)** — consequence of IV-2

| Tests | Error | Node | DSL fn |
|-------|-------|------|--------|
| 26 | `cannot compare Str("e0") with Int(768)` | `std.render::span_width`, `std.unicode::string_display_width`, `std.width::truncate_text` | Call `code_point(c)` → compare with block ranges |
| 13 | `no match arm matched value: Map({"value": Enum})` | `std.unicode::char_width` | `char_display_width` returns `DisplayWidth` variant, `char_width` matches on it |

Root cause: `Char = Int where brand("Char")`. Mock generates `Str` instead
of `Int`. Fn body does `code_point(c)` → Int, then `>=` comparison fails.
The 13 no-match errors are downstream: `char_display_width` produces a
variant wrapped in `Some(value)` Map, match patterns don't match that form.

**Group B: ResourceHandle WrapScalar coercion (34 tests)**

| Tests | Error | Pattern |
|-------|-------|---------|
| 34 | `WrapScalar should deliver list; got Map({cap, key, resource_id})` | `acquire_resource_*` → `release_resource_*` across 7 modules |

Root cause: ResourceHandle product witness produces `Map({cap, key,
resource_id})`. The WrapScalar coercion should wrap it into a List but
doesn't. Likely because the product witness omits the `type` field
(the real ResourceHandle includes `type: "resource_handle"` marker)
or because the coercion type check rejects the new Map shape.

**Group C: List-where-scalar-expected (21 tests)**

| Tests | Error | Node |
|-------|-------|------|
| 11 | `field 'name' on List([Map({name, description})])` | `tools.readme::render_readme` |
| 4 | `field 'text' on List([Map({style, text})])` | `std.render::truncate_spans` |
| 3 | `field 'success' on List([Map({name, success, ...})])` | `shared.dag_util::aggregate_results`, `format_report` |
| 2 | `field 'kind' on List([Map(...)])` | misc |
| 1 | `field 'when_build_system' on List` | `extdeps.gitignore_render` |

Root cause: The mock value is a List containing correct Maps, but the
fn body accesses `.field` on the List itself. The mock wrapping is wrong
— the port type says `List<ProductType>` but the fn parameter expects
a scalar `ProductType`. Cardinality inference may be misattributing.

**Group D: Nested product field resolution (21 tests)**

| Tests | Error | Node |
|-------|-------|------|
| 8 | `field 'indent' on Str("mock")` | `std.render::constrain_frame` |
| 5 | `field 'source' on Str("mock")` | `extdeps.gitignore_render::render_gitignore_file` |
| 4 | `field 'is_blank' on Str("mock")` | `shared.dag_util::render_document_section` |
| 4 | `field 'has_title' on Str("mock")` | `shared.dag_util::render_document_section` |

Root cause: Product types containing fields that are themselves product
types (e.g., `Frame.lines: List<Line>` where `Line.indent: Int`). The
outer product witness generates correct values for scalar fields, but
nested product fields get `Str("mock")`. This is IV-1 again: the
recursive `typed_witness_value_depth` call for inner types uses
`resolve_type` which may fail for certain type names, falling back
to the placeholder.

#### Relationship to FC phases

The 115 failures map directly to the FC phase plan:

- **Phase A** (tighten construction): Fix IV-1 (`resolve_type` must handle
  all registered names) and IV-2 (`base_type` must see through brands).
  This eliminates Groups A and D (60 tests) by making the type system
  return correct data instead of fabricated placeholders.

- **Phase B** (convert fabrications to errors): Make `scalar_witness_for_base`
  return `Err` for unknown bases instead of `Str("<Type>")`. Make
  `value_backing_for_type_id` return `Err` instead of `Json`. This surfaces
  the remaining issues as compile-time errors instead of silent mock
  degradation.

- **Phase C** (remove escape hatches): Delete `is_placeholder_witness` filter,
  `CallableOp` unbound-variable fallback, and `get_by_name` before
  `resolve_type` workaround. Once upstream fabrications are gone, these
  compensating patches become dead code.

- **Phase D** (wire clean path): With correct types flowing through, Group B
  (ResourceHandle coercion) and Group C (cardinality) either resolve
  naturally or surface as clear, actionable errors.

### 8. Match-arm service call branch isolation (compiler bug)

The DSL lowerer (`daglang-lower/src/lib.rs:4725-4729`) does not isolate
service calls within match arms into their respective branches. When a
`match` expression routes to different services (e.g., OpenAI vs Anthropic),
all arms' transport triplets end up in BOTH branches. In tests (where all
transports are mocked), both branches execute and produce real values,
causing the merge node to reject duplicate scalar values.

**Impact:** Any `match` with service calls in multiple arms fails in
dry-run/test execution. The deleted `tools.design` module was the only
consumer. No current DSL module uses this pattern.

**Fix:** Route the match condition into the BranchBuilder's condition port
so only the matching arm's transports execute. The infrastructure exists
(`BranchBuilder::condition`), but the match-to-branch lowering path doesn't
wire it.

---

## FC-7: PortMultiplicity was a degenerate abstraction (2026-03-10)

### The problem

The `Port` struct had three fields encoding related concerns:

| Field | Purpose | Example |
|-------|---------|---------|
| `type_id` | What type of value (`"String"`, `"List<String>"`) | — |
| `cardinality` | How many elements (`[1,1]`, `[0,∞)`) | — |
| `multiplicity` | How many edges (`Singular`, `FanIn`) | — |

`cardinality` was already derived from `type_id` by the builder (via
`TypeRegistry::infer_cardinality`). `multiplicity` was a separate boolean
that told the executor whether to merge values from multiple edges into
a list.

The `FanIn` variant was used by exactly 4 ports in the entire system:
`__deps` (×3) and `required_scopes` (×1). All infrastructure, none
user-visible. But its existence created a class of bugs where cardinality
was set correctly but multiplicity was not (or vice versa), producing
silent behavioral differences.

### The symptom

Resource release nodes had `cardinality = ZERO_OR_MORE` on their
`resource_handle` input (correctly — multiple acquire nodes can feed
handles), but `multiplicity = Singular` (the default). The executor
treated the port as scalar, not fan-in, so WrapScalar coercion tests
failed: the value arrived as a raw `Map` instead of `List([Map])`.

### The analysis

Tracing the executor's input-gathering algorithm revealed the
multiplicity check was doing exactly what `cardinality.is_list()` would
do. The two concepts collapsed:

- `cardinality.is_list()` → collect values from N edges into a list
- `!cardinality.is_list()` → take one value (conditional merge for branches)

The `multiplicity` field was encoding the same information as cardinality
at a different level of abstraction — it was literally "cardinality of
cardinality." The `window.rs` test harness had already independently
discovered this: it used `port.cardinality.is_list()` instead of
`port.multiplicity == FanIn`, and worked correctly.

### The fix

Deleted `PortMultiplicity` entirely. The executor now uses
`port.cardinality.is_list()` to determine merge strategy:

```rust
// Before: two fields, easy to get out of sync
let fan_in_ports: HashSet<&str> = node.inputs.iter()
    .filter(|p| p.multiplicity == PortMultiplicity::FanIn)  // ← separate flag
    .map(|p| p.name.0.as_str()).collect();

// After: one field, derived from type
let list_ports: HashSet<&str> = node.inputs.iter()
    .filter(|p| p.cardinality.is_list())  // ← same information, no duplication
    .map(|p| p.name.0.as_str()).collect();
```

`Port::fan_in()` was replaced by `Port::list()` (which already existed).
All 4 callsites updated. Zero behavioral change — the algorithm is
identical, just driven by one field instead of two.

### The lesson

When a second field exists only to tell the runtime "actually, pay
attention to the first field," the two fields encode the same dimension.
The right response is to delete the second field and make the first one
authoritative.

More broadly: if you need N concepts to express a behavior, and N-1 of
them can be derived from the remaining one, you have N-1 too many
concepts. The complexity budget should go toward making the one
authoritative concept clear, not toward keeping multiple representations
in sync.
