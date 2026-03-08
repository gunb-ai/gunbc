# POSTMORTEM

Rolling postmortem — technical debt and incidents identified during development.
Each item has enough context to be picked up cold.

**Origin:** T1–T8 identified during the generated test fix-up (2026-03-08).
T9–T11 identified during the `make gist` credential failure investigation (2026-03-08).
T12 identified during the silent `Error 1` investigation (2026-03-08).
T13 identified during the zero-fallback audit (2026-03-08) — subsumes T3, T5, T6, T9, T10, T12.

**Policy:** These items are interconnected. Do not spot-fix individual items —
they need to be considered holistically and designed together before any
implementation begins. T13 (zero-fallback) is the root cause of 6 items and
amplifies 3 more. The remaining independent items (T1, T4, T8) have their own
root causes. A spot fix in one area will shift the failure to another. The
goal is a single cohesive design pass that resolves the underlying structural
issues.

**Clusters:**

- **Zero-fallback policy** (T13 — subsumes T3, T5, T6, T9, T10, T12): The compiler has ~70 sites across 7 layers where it silently degrades instead of failing. This is the root cause of most items in this document. T3, T5, T6, T9, T10, and T12 are all specific symptoms of the same pattern: a fallback produces valid-looking but wrong output, which cascades into a confusing runtime failure or silent exit. One design: remove all lossy/fallback/warning paths from the compiler. Every compilation path either succeeds fully or fails with a clear error. See T13 for the full taxonomy.
- **Auth & resource architecture** (T7, T8): No concrete Filesystem binding + hand-rolled credential materialization. One design: credential provider interface (T8) with concrete transport bindings for Filesystem (T7), resolved via execution profiles. T7's silent Skipped cascade is amplified by the fallback pattern (T13).
- **Executor semantics** (T1, T4): Non-deterministic fan-in + ad-hoc conditional merge. One design: deterministic edge ordering (T1) + formal ConditionalMerge node (T4), making execution order explicit in the IR.
- **Type system gaps** (T2): Secret not first-class. One design: complete the ValueBacking model. The `IsString` accepting `Secret` workaround is a type-layer fallback (T13).
- **Test coverage model** (T11): DryRun-only tests hide all of the above. One design: tiered execution with selective mocking, virtual backends for testable I/O, Real-mode tests for pure subgraphs. DryRun passes for every item in this document because the fallback pattern (T13) produces valid-looking output.

---

## T1: Deterministic fan-in ordering in executor

**Priority: High**
**Files:** `src/core/exec/src/execute/mod.rs`

The executor processes fan-in edges (multiple edges targeting the same list port) in topological-sort order. The topological sort is non-deterministic for nodes at the same depth — it depends on HashMap iteration order and node insertion order. This means `Value::List` outputs built from fan-in have unstable element ordering across runs or between a full DAG and a subDAG extraction.

**Current workaround:** `assert_window_outputs` in `src/core/test/src/window.rs` accepts any matching-length list instead of checking value equality. This is extremely permissive.

**Proper fix:** When the executor collects fan-in values for a list port, sort by a deterministic key before constructing the `Value::List`. Candidates:
- Edge index (already stored in `Edge.index`, but currently defaulting to 0 for all edges)
- `(from_node, from_port)` lexicographic order
- Assign monotonic indices during lowering so edges carry stable ordering

After fixing, revert the matching-length comparison in `window.rs` back to exact equality (or multiset comparison).

**Test:** The 32 window tests in `tools_build` and `workflows_build_all` that go through `expr_compute_*_stages_arg_*` nodes — these currently pass only because of the lenient comparison.

---

## T2: Secret as first-class ValueBacking

**Priority: Medium**
**Files:** `src/core/ir/src/types.rs`, `src/core/test/src/auto_mock.rs`, `src/core/test/src/mock_spec.rs`, `src/core/codegen/src/testgen/codegen.rs`

`ValueBacking` has no `Secret` variant. `value_backing_for_type_id("Secret")` returns `ValueBacking::String`, so the auto-mock generates `OutputMatcher::IsString` for Secret-typed ports. But the runtime value is `Value::Secret(...)`, not `Value::Str(...)`.

**Current workaround:** Two places paper over this:
1. `OutputMatcher::IsString` check in `mock_spec.rs` accepts `Value::Secret(_)` alongside `Value::Str(_)`
2. The codegen renders `matches!(*v, Value::Skipped | Value::Secret(_))` in the IsString assertion

**Proper fix:**
1. Add `ValueBacking::Secret` to the enum
2. Register it in the type registry for `TypeId("Secret")`
3. Add `OutputMatcher::IsSecret` variant
4. Auto-mock emits `IsSecret` for Secret-typed ports
5. Remove the Secret accommodations from IsString
6. Consider whether the DSL coercion system should handle String->Secret upcast (safe) and Secret->String downcast (explicit only)

---

## T3: Testgen-side filter for unevaluable fn bodies

**Subsumed by T13** — `ExprComputeOp` degrading to `Skipped` on unknown functions is a resolver-layer fallback.

**Priority: Medium**
**Files:** `src/core/codegen/src/testgen/codegen.rs`, `src/core/resolve/src/resolve.rs`

Some fn bodies in `.dag` files contain service calls (e.g., `llm.Anthropic.Messages()` in `design.dag`). The `ExprComputeOp` evaluator can't execute these — it hits "unknown function" errors.

**Current workaround:** `ExprComputeOp::execute()` in `resolve.rs` catches errors containing "unknown function" and degrades to `Value::Skipped` instead of returning an error. This silently swallows real errors alongside the expected ones.

**Proper fix:** Detect unevaluable fn bodies at testgen time and skip generating example tests for those nodes. Detection approach:
- During lowering, `ExprComputeOp` already stores the fn body AST
- Walk the AST for service call expressions (identifiable by module-qualified names like `llm.Anthropic.Messages`)
- If found, mark the node as unevaluable in the `DagAnalysis`
- Testgen skips example/chain tests for unevaluable nodes

Then remove the silent "unknown function" fallback in `ExprComputeOp::execute()` so the executor stays strict.

---

## T4: Scalar fan-in semantics for conditional branches

**Priority: Low**
**Files:** `src/core/exec/src/execute/mod.rs`

When conditional branches (if/match lowered to CondBranch) produce edges to the same scalar port, the executor now accepts multiple upstream edges and takes the first non-Skipped value. Previously it rejected this as a "duplicate scalar port" error.

**Current behavior:** In both sequential and parallel executors (~line 655 and ~line 1250), when a scalar port already has a source and a new edge arrives:
- If the new value is non-Skipped, it overwrites the existing value
- If the new value is Skipped, the existing value is kept

This works for the current pattern (exactly one branch fires, others produce Skipped) but is ad-hoc.

**Proper fix:** Introduce a formal `ConditionalMerge` node kind in the IR that explicitly merges conditional branch outputs. The lowerer would emit this node at the branch join point. The executor would have well-defined semantics: exactly one non-Skipped input expected, error if zero or 2+. This makes the behavior visible in the DAG structure rather than implicit in the executor's edge processing.

---

## T5: Literal source terminal example generation

**Subsumed by T13** — skip-by-prefix workaround masks fn body evaluation fallback producing wrong values.

**Priority: Low**
**Files:** `src/core/test/src/auto_mock.rs`

`literal_source_*` nodes (created by the lowerer for fn body literal expressions) are skipped from terminal example generation because their fn body evaluation can produce empty values that fail the `NonEmpty` fallback matcher.

**Current workaround:** `auto_mock_spec` skips all nodes whose ID starts with `literal_source_` when generating terminal node examples.

**Proper fix:** Instead of skipping by name prefix, the auto-mock should trial-execute the node with its example inputs and verify matchers pass before emitting the example. The previous attempt at this (blanket trial validation) was too aggressive — it removed ~360 valid examples because many trial executions fail for unrelated reasons (missing context, upstream dependencies). A more targeted approach:
- Only trial-validate when the fallback `NonEmpty` matcher is used (not typed matchers like IsBool/IsInt)
- Accept the example if the trial execution fails (the test may work differently)
- Only skip if the trial succeeds AND the matcher explicitly fails on the produced value

---

## T6: Parser: multi-statement blocks in fn body expression position

**Subsumed by T13** — `parse_body_lossy()` is the canonical lossy fallback: parser returns empty body instead of failing.

**Priority: Medium**
**Files:** `src/core/daglang/daglang-syntax/src/parser.rs`

`parse_fn_body_lossy()` falls back to lossy parsing when the fn body contains multi-statement blocks in expression position (e.g., `let` bindings inside `if` branches). The lossy body has `fn_body: None` at resolve time, so the fn item resolves as `DeclaredOutputCallableOp` (identity passthrough) instead of `FnBodyCallableOp`. This causes a runtime error: "missing required declared output passthrough: `return`".

**Current workaround:** DSL authors must hoist `let` bindings out of `if`/`match` branches in `fn` items. Example from `tools/gist.dag`:
```
// BROKEN: let inside if branch causes lossy parse
let skip_section = if skipped |> count() > 0 {
    let skip_lines = skipped |> map(s => "- {s}") |> join("\n")
    "\n\n## Skipped\n\n{skip_lines}"
} else { "" }

// FIXED: hoist let before if
let skip_lines = skipped |> map(s => "- {s}") |> join("\n")
let skip_section = if skipped |> count() > 0 {
    "\n\n## Skipped\n\n{skip_lines}"
} else { "" }
```

**Proper fix:** `consume_brace_block_expr()` in the parser should handle multi-statement blocks in expression position — parse `let` bindings + final expression as a block expression (similar to Rust's `{ let x = ...; expr }`). This would also unblock lambdas with `let` bindings inside fn bodies, which currently produce empty Records.

**Diagnosis:** When `parse_fn_body()` fails or adds errors, `parse_body_lossy()` resets position and calls `consume_brace_block_contents()`, discarding the entire body and marking `lossy: true`. The lowerer then skips fn_body collection for lossy bodies (line ~2415 in `daglang-lower/src/lib.rs`).

**Detection gap:** Lossy fn bodies are tracked in `CompileOutput.lossy_fn_bodies` but only for informational purposes — no warning or error is emitted. Adding a compile-time warning for lossy fn bodies would catch this class of bug earlier.

**Incident — `make gist` credential failure (2026-03-08):** `github_token()` in `auth.dag` has a multi-statement `else` branch (three assignments + final expression) that triggers lossy parsing. With `fn_body: None`, the function resolves as `DeclaredOutputCallableOp`. The return `{ token: token }` references a local variable assigned from the `if/else`, which `resolve_return_expr_source` cannot trace (see T9). `__out:token` is never wired → `Value::Skipped` → flows to `res:credential` on `github.Gist.Create` → transport rejects it: `"expected Credential, Secret, or String, got Skipped"`. This happens regardless of whether `GITHUB_TOKEN` is set — the env var is read correctly by `shell.Env.Get` but the result is lost at the function boundary.

---

## T7: Filesystem interface has no concrete binding (Real mode broken)

**Priority: High**
**Files:** `dsl/extdeps/github/auth.dag`, `dsl/std/patterns.dag`, `src/core/resolve/src/service_ops/service_ops_impl.rs`

`InterfaceStubExecuteOp` always errors in Real mode with "has no concrete binding". This means any `func` that declares `uses fs: Filesystem(...)` unconditionally fails in Real mode — even if the code path that actually uses Filesystem capabilities is never reached.

**Impact:** Resource acquisition runs unconditionally at DAG execution start. If the acquire node errors, `GenericRestPrepareOp` sees Skipped inputs downstream and skips the entire prepare → execute → parse chain, producing empty output. This caused `make gist` to silently produce empty results despite dry-run passing.

**Current workaround:** Removed `uses fs: Filesystem(mode: Read)` from `github_token()` in `auth.dag`. The ADC fallback path (reading `~/.config/gcloud/application_default_credentials.json`) is now unreachable — `github_token` only works via the `GITHUB_TOKEN` env var. Any DSL tool using `read_text_files` or `classify_files` from `std/patterns.dag` also fails in Real mode for the same reason.

**Proper fix:** Implement a concrete Filesystem transport binding (file read/write/probe via actual I/O), or make resource acquisition conditional so unused resources don't block execution.

**Affected patterns:**
- `std.patterns.read_text_files` — uses `Filesystem.probe` + `Filesystem.read`
- `std.patterns.classify_files` — uses `Filesystem.probe`
- Any `func` with `uses fs: Filesystem(...)` in its signature

---

## T8: Auth layer separation — services should declare needs, not materialization

**Priority: Medium**
**Files:** `dsl/extdeps/github/auth.dag`, `dsl/extdeps/cloud/gcp/gcp.dag`

`github_token()` in `auth.dag` mixes two concerns: what GitHub needs (a token with `gist` scope) and how to get it (env var, ADC refresh, Secret Manager). This is a GitHub-specific credential function, but the materialization logic (env → ADC → Secret Manager) is generic and will be duplicated for every service that needs a secret.

**Current state:** Each service consumer manually calls `github_token()` and threads the token through. The service definition (`gists.dag`) takes `auth_token: Secret` as an explicit input. If we add a second GitHub service (issues, PRs), each caller must independently acquire and pass the token.

**Proper fix:** Follow the same interface/binding pattern used for resources:

1. **Service declares requirements**: `github.Gist.Create` declares "I need a credential with `gist` scope" as a capability requirement, not an explicit input field
2. **Credential provider interface**: Generic `CredentialProvider` interface with operations like `resolve(scope) -> Secret`. Concrete bindings: `EnvVarProvider` (reads env), `GcpSecretManagerProvider` (ADC + refresh + Secret Manager), etc.
3. **Runtime resolution**: The executor resolves credential requirements to a provider based on execution environment (local dev → env var, CI → GCP Secret Manager), similar to how resource interfaces get concrete bindings
4. **Service config binding**: The `auth: BearerToken` + `auth_input: auth_token` fields in service config would reference the credential provider instead of an explicit input port

**Benefits:**
- GitHub service definitions only declare what scopes they need
- Credential materialization is defined once, reused across all services
- Environment-specific auth strategies are configured at the profile level (unit_test / local / cloud_run), matching the existing profile model in `dsl/sdlc/profiles/`
- No more threading `auth_token` through every caller

---

## T9: RT4c — `resolve_return_expr_source` cannot wire local variable references

**Subsumed by T13** — `lower_warn` + `continue` on unwired output is a lowerer-layer fallback.

**Priority: High**
**Files:** `src/core/daglang/daglang-lower/src/lib.rs`

`resolve_return_expr_source` (line ~10163) resolves return expressions to DAG node outputs for wiring `__out:*` passthrough ports. It checks five source maps: `param_types`, `bound_callable_sources`, `bound_service_sources`, `expanded_results`, `endpoints_by_name`. It does not track local variable assignments — identifiers bound by `let`/assignment statements inside fn/func/pattern bodies.

When a return expression references a local variable (e.g., `return { token: token }` where `token = if ... { ... } else { ... }`), the lookup fails and `wire_callable_return_outputs` emits an RT4c warning (line ~11881) then skips the wiring. The `__out:*` port remains unwired. Since `DeclaredOutputCallableOp` marks all outputs as optional (line ~1522 in `resolve.rs`), the unwired port silently produces `Value::Skipped`.

**Relationship to T6:** T9 only manifests when T6 forces lossy parsing (`fn_body: None`), which activates passthrough mode and requires return wiring. If the parser handled multi-statement blocks (T6 fix), `FnBodyCallableOp` would evaluate the body directly and T9 would not apply. Conversely, fixing T9 without fixing T6 would also resolve the gist failure — the lowerer would wire through the local variable to the underlying conditional/service-call nodes.

**Current workaround:** None. The RT4c warning is gated behind `DAGLANG_LOWER_WARNINGS=1` (off by default). The function silently produces `Value::Skipped` for all outputs.

**Proper fix:** Track local binding sources during lowering. When `wire_fn_call_arguments` processes statements, build a map of `assignment_name → (source_node_id, source_port)` for each `let`/assignment whose RHS resolves to a known node output (service call, callable, conditional). `resolve_return_expr_source` would then check this map for `Expr::Ident` lookups after the existing five maps. For compound RHS expressions (conditionals, pipes), the lowerer already synthesizes structural nodes (via `synthesize_conditional`, `synthesize_binary_op`, etc.) — the local binding map just needs to record the synthesized node's output.

**Incident:** See T6 incident note. `github_token().token` always returns `Value::Skipped` due to this wiring gap, breaking `make gist` even when `GITHUB_TOKEN` is correctly set.

---

## T10: Promote `lower_warn` diagnostics to compile errors

**Subsumed by T13** — T10 catalogs 8 fallback sites in the lowerer/testgen; T13 expands this to all 7 layers.

**Priority: High**
**Files:** `src/core/daglang/daglang-lower/src/lib.rs`, `src/core/test/src/auto_mock.rs`, `src/core/codegen/src/testgen/codegen.rs`

The compiler has several "soft warning" paths that silently degrade output instead of failing. This contradicts the project's error-or-success philosophy (see invariant I8: "Warnings are errors. If something is wrong, the build fails.").

**`lower_warn` (4 call sites, off by default):**

The `lower_warn` function (line ~54) is gated behind `DAGLANG_LOWER_WARNINGS=1` — disabled by default. Four call sites silently drop or skip work:

1. **Pattern node expansion failure** (line ~5580): Node dropped from expanded pattern, no error.
2. **Non-ident base in field access return** (line ~5672): Return field unresolved, silently skipped.
3. **Unsupported expression type in pattern body** (line ~5846): Node silently skipped — only ServiceCall, eq(), and pattern calls supported.
4. **RT4c: Return output can't be wired** (line ~11888): `__out:*` port left unwired, producing `Value::Skipped` at runtime. This is the direct cause of the `make gist` credential failure.

**Other silent degradation:**

5. **`auto_mock.rs` line ~555**: `probe_best_response` falls back to `default_shell_response()` when all candidates fail — prints to stderr but continues with a potentially wrong mock.
6. **`testgen/codegen.rs` line ~5659**: Corpus identity not found in DAG — warns about drift but generates tests against stale data.
7. **`testgen/codegen.rs` line ~5705**: Effectful node with ExactOutputs silently falls back to DryRun mode instead of the requested Real mode.
8. **`CompileOutput.lossy_fn_bodies`** (`daglang-driver/src/lib.rs` line ~63): Lossy fn bodies are collected but never surfaced — no error, no warning, nothing checks the list. (Also noted in T6's detection gap.)

**Proper fix:** Each of these should either be a hard compile/lower error or should be explicitly opted into via a `#[allow(...)]`-style mechanism:
- `lower_warn` call sites 1–3: Return `Err` from the lowering function so the compilation fails with a clear diagnostic.
- `lower_warn` call site 4 (RT4c): Return `Err` — an unwired output that silently becomes Skipped is always a bug.
- `lossy_fn_bodies`: Check the list after compilation and fail if non-empty (or require explicit `#[lossy]` annotation on fn items that are known to need it).
- Auto-mock and testgen warnings: Fail test generation with an actionable error instead of silently degrading.

Delete the `lower_warn` function and the `DAGLANG_LOWER_WARNINGS` env var gate entirely.

---

## T11: DryRun-only test coverage hides Real-mode failures

**Priority: High**
**Files:** `src/core/codegen/src/testgen/codegen.rs`, `src/core/exec/src/execute/mod.rs`, `src/core/test/src/auto_mock.rs`

Every generated test for every `.dag` module runs in `ExecutionMode::DryRun(mocks)`. DryRun intercepts all transport-execute nodes and boundary-mocked nodes before they execute, substituting mock outputs. This means the test suite proves DAG *structure* (edges, ports, mock compatibility) but never tests whether the DAG *executes correctly* in Real mode.

**Incident — why every postmortem item survives `make test-all`:**

| Item | Why DryRun passes | What Real mode would catch |
|------|-------------------|---------------------------|
| T1 | Window tests use lenient `a.len() == b.len()` comparison | Non-deterministic fan-in ordering |
| T2 | `IsString` matcher accepts `Value::Secret(_)` | Secret/String type confusion |
| T3 | `ExprComputeOp` catches "unknown function" → Skipped | Unevaluable fn bodies silently degrade |
| T4 | Executor silently merges scalar fan-in | Ad-hoc conditional merge semantics |
| T5 | `auto_mock` skips `literal_source_*` by prefix | Literal nodes produce unexpected values |
| T6/T9 | `github_token` is a boundary mock — never executes | Lossy body + unwired `__out:token` = Skipped |
| T7 | `InterfaceStubExecuteOp` is auto-mocked in DryRun | Always errors in Real mode ("no concrete binding") |
| T10 | `lower_warn` is off by default — invisible to everyone | Silent lowering failures produce broken DAGs |

**Root observation:** DryRun mocking is too coarse. It replaces entire node execution with canned outputs, which is correct for external I/O (HTTP calls, shell commands) but wrong for nodes that contain testable internal logic (credential resolution, conditional branching, type coercion). Not everything needs a mock — many operations are pure or can run against virtual/in-process backends.

**What can run in Real mode without external I/O:**

1. **Env var reads** (`shell.Env.Get`): The shell transport runs `printenv`. Instead of mocking the entire node, inject the env var into the test process environment and let the real transport execute. This is a `std::env::set_var` call — zero I/O.

2. **Filesystem operations** (`Filesystem.read`, `Filesystem.probe`): Use a virtual filesystem or tempdir. The `ResourceAcquireOp` for Filesystem already returns a `FilesystemHandle` — wire it to an in-process VFS backend instead of stubbing the entire capability chain.

3. **Conditional logic** (`if`/`match` branches, credential selection): Pure computation. No reason to mock — let the executor evaluate the real branch logic and verify the correct branch fires.

4. **Type coercions** (`as Secret`, field access, string interpolation): Pure. `ExprComputeOp` and structural nodes should execute in Real mode for these.

5. **Resource lifecycle** (acquire/release): `ResourceAcquireOp` for Network, Clock, AuthContext returns literal strings — no I/O. Only Filesystem capability calls need a backend.

**What must stay mocked (actual external I/O):**

- REST transport calls (`github.Gist.Create`, `oauth2.Google.Refresh`, `gcp.SecretManager.AccessVersion`)
- Shell commands that modify system state
- Cloud API calls

**Proper fix — tiered test execution:**

The test framework already has the infrastructure for this. `ExecutionMode::DryRun(BoundaryMocks)` intercepts nodes based on `should_intercept_for_mode`. The fix is to narrow interception scope:

1. **Selective mocking**: Instead of mocking all boundary nodes, only mock nodes that perform real external I/O (transport-execute nodes with REST/HTTP/shell transports that hit external systems). Let internal logic nodes execute normally.

2. **Virtual backends for testable I/O**: For shell.Env.Get, provide a test env var map. For Filesystem operations, provide a VFS. The transport layer already dispatches by transport type (`TransportRequest::Rest`, `TransportRequest::Shell`, etc.) — add `TransportRequest::VirtualFs` or inject a test backend at the transport executor level.

3. **Real-mode generated tests for pure subgraphs**: Testgen should identify subgraphs that contain no external I/O (or only virtualizable I/O) and generate Real-mode tests for them. The `is_pure_node` check in corpus test generation (line ~5699 in `codegen.rs`) already classifies nodes by purity — extend this to subgraph-level purity analysis.

4. **Credential flow integration test**: For `auth.dag` specifically, a Real-mode test that sets `GITHUB_TOKEN` in the process environment and verifies `github_token().token` produces a non-Skipped `Value::Secret` would have caught T6/T9 immediately. This test needs zero network access.

**Test gap metric**: Of the ~24 nodes in the gist DAG, only the 3 REST transport-execute nodes (Gist.Create, and 2 git operations that happen to use shell transport) need external mocking. The remaining ~21 nodes — conditional logic, env var reads, string interpolation, field access, resource lifecycle — could run in Real mode with no I/O.

---

## T12: Error rendering architecture — silent exit on Skipped cascade

**Subsumed by T13** — distributed rendering with implicit "already rendered" assumptions is a display-layer fallback.

**Priority: High**
**Files:** `src/core/exec/src/display.rs`, `src/core/codegen/src/cli_gen.rs`

The error rendering architecture has multiple independent decision points with no central authority on whether an error has been shown to the user. When these decision points disagree, the result is `process::exit(1)` with no user-visible diagnostic.

**Incident — silent `Error 1` from `make gist` (2026-03-08):**

`make gist` exited with code 1 but displayed no error message. The output showed `🐧 Completed [49ms]` and `✅ 17 completed — 17/24 [49ms]` — the DAG phase was `Completed`, not `Failed`. Seven nodes were skipped (shown as `○`) because `Value::Skipped` propagated from the unwired `github_token()` output (T6/T9). The executor returned `Err(ExecError)` with a `NodeTrace` layer. Two rendering paths both declined to show the error:

1. **`print_error_boxes`** checks `progress.failed_nodes()` — finds zero nodes in `NodeState::Failed` (skipped nodes are `NodeState::Skipped`). Prints nothing.
2. **`should_render_fallback_error`** checks `err.node_trace().is_none()` — the error HAS a `NodeTrace` layer, so it assumes the error was "already rendered" by error boxes. Returns false. Prints nothing.

Net result: `process::exit(1)` with zero diagnostic output.

**Root cause — distributed rendering with implicit "already rendered" assumptions:**

The error rendering architecture has four independent rendering paths, none of which tracks whether any other path actually rendered:

| Path | What renders | Decision heuristic |
|------|-------------|-------------------|
| **Progress observer** (`NonTtyProgressObserver`) | `❌ Failed at node_id: ...` during execution | Renders if executor calls `on_node_failed` |
| **Error boxes** (`print_error_boxes`) | Structured `╭─ ... ─╮` boxes after completion | Renders if `progress.failed_nodes()` is non-empty |
| **Fallback** (`should_render_fallback_error`) | Generic `print_attention("Execution failed", ...)` | Renders if error has no `NodeTrace` layer |
| **Success port** | "A required success check returned false." | Renders if `success_port_failed` returns true |

The problem: path 3 assumes that if a `NodeTrace` is present, paths 1 or 2 already rendered. But paths 1 and 2 key on `NodeState::Failed`, while `NodeTrace` can be attached to errors from nodes that are `Skipped` in the progress tracker. When the progress state diverges from the error state, all four paths produce nothing.

**Immediate fix (applied):**

- Removed `should_render_fallback_error` heuristic entirely. `execute_and_display`'s `Err` path now always calls `print_attention`. This may produce mild redundancy when error boxes also rendered, but redundancy is better than a diagnostic black hole.
- Fixed generated step-mode code in `cli_gen.rs` which had a `process::exit(1)` on success-port-false with no message.

**Additional gap found during audit:**

- `ChannelObserver` send failures (line ~547 in `display.rs`): if the display channel disconnects before a `NodeFailed` event is received, structured error box context is lost. The flat error is still printed via the fallback, so this is not silent, but structured context (service, HTTP, auth layers) is lost.

**Proper fix — centralized error rendering:**

The fundamental issue is that "has this error been shown?" is implicit, distributed across four paths that use different heuristics. This should be explicit and centralized:

1. **`ErrorRendered` token**: Introduce a `rendered: bool` (or `AtomicBool` for the parallel path) threaded through all rendering paths. Each path that prints sets `rendered = true`. The final `process::exit(1)` path checks: if `!rendered`, always print the full error.

2. **Single rendering chokepoint**: Instead of four independent paths, funnel all errors through a single `render_execution_error(err, progress, log)` function called once in `execute_and_display`. This function inspects the error, the progress state, and the log to decide what to render — structured box, flat message, or both. No path makes independent rendering decisions.

3. **Progress/error state alignment**: When the executor returns `Err` with a `NodeTrace`, the progress tracker should mark that node as `Failed` (not leave it as `Skipped`). This ensures `print_error_boxes` finds the failure. The `DagProgress::apply` event model already supports `NodeFailed` — the gap is that the executor's error return may not have been preceded by an `on_node_failed` call (e.g., if the error originated in post-processing after the node was marked Skipped).

**Relationship to other items:** T12 is a display-layer manifestation of T10's broader "silent degradation" theme. T10 covers silent degradation at compile/lower time; T12 covers silent degradation at display/exit time. Both stem from the same design gap: systems that silently decline to report problems instead of failing loudly.

---

## T13: Remove all fallback/lossy/warning behavior from the compiler

**Priority: Critical**
**Subsumes: T3, T5, T6, T9, T10, T12**
**Amplifies: T2, T7, T11**

The compiler was intended to have a strict zero-fallback policy: every compilation path either succeeds fully or fails with a clear error. An audit found ~70 sites across 7 layers where the compiler silently degrades instead of failing. This is the root cause of most items in this document.

**The pattern:** A fallback produces valid-looking but wrong output. The wrong output flows through the DAG as `Value::Skipped`. Downstream nodes propagate the Skipped value. Eventually, a transport rejects the Skipped value with an error message that points to the consumer, not the producer. Or worse — all rendering paths decline to show the error, and the process exits silently.

**Why this is one task, not six:** T3, T5, T6, T9, T10, and T12 are all specific instances of the same design gap. Fixing any one of them individually shifts the failure to the next fallback in the chain. The fix is to remove the fallback pattern itself: make every layer fail loudly on invalid input, and remove the machinery that produces "valid-looking but wrong" output.

### Layer 1 — Parser lossy fallback (6 sites)

`parse_body_lossy()` in `daglang-syntax/src/parser.rs` (line ~2830) is the entry point. When `parse_fn_body()` or `parse_func_body()` fails or produces errors, the parser rewinds to the start of the block, discards all errors from the attempt, calls `consume_brace_block_contents()` to skip past the `}`, and returns a body with `stmts: Vec::new()` and `lossy: true`.

Called for: fn bodies (line 1882), func bodies (line 1910), pattern bodies (line 1933), resource acquire/release (lines 2168/2174), stage bodies (line 2502).

Additionally, `consume_brace_block_expr()` (line 3523) handles multi-statement blocks in expression position (if/match arms). It parses `stmts` but always returns `Expr::Record(None, Vec::new())`, discarding all content. This is the root cause of T6.

The driver collects lossy fn bodies in `CompileOutput.lossy_fn_bodies` (line ~63 in `daglang-driver/src/lib.rs`) but never surfaces them as errors or warnings.

**Fix:** `parse_body_lossy` should be deleted. If the parser can't parse a body, it must return `Err`. `consume_brace_block_expr` should parse block expressions properly (Expr::Block with let bindings + final expression) or return `Err`. `lossy_fn_bodies` should be checked after compilation and treated as a compile error.

### Layer 2 — Lowerer silent skips (~30 sites)

The lowerer in `daglang-lower/src/lib.rs` uses `continue` extensively when it can't resolve endpoints, clone loop bodies, expand patterns, wire arguments, or resolve return expressions. Most of these produce no diagnostic at all. Four sites use `lower_warn()` (line ~54), which is gated behind `DAGLANG_LOWER_WARNINGS=1` — disabled by default.

The four `lower_warn` sites (documented in T10):
1. Pattern node expansion failure (line ~5580) — node dropped
2. Non-ident base in field access return (line ~5672) — return field unresolved
3. Unsupported expression type in pattern body (line ~5846) — node skipped
4. RT4c: Return output can't be wired (line ~11898) — `__out:*` port unwired → `Value::Skipped`

Additional silent `continue` sites (~25):
- `endpoints_by_name.get` returns `None` → skip call/wiring (~12 sites)
- Loop body cloning/resolution fails → skip (~3 sites at lines 4150, 4190, 4196)
- Auth/resource wiring fails → skip (~3 sites at lines 8650, 8742, 8815)
- Service endpoint resolution fails → skip (~5 sites)
- Pattern expansion fails → skip (~2 sites)

**Fix:** Delete `lower_warn` and the `DAGLANG_LOWER_WARNINGS` env var. Every `continue` that drops a node, edge, or wiring from the DAG should return `Err` from the lowering function. The compilation should fail with a diagnostic that names the specific construct that couldn't be lowered.

### Layer 3 — Resolver degradation (~10 sites)

The resolver in `resolve.rs` produces `Value::Skipped` for several failure modes that should be errors:
- `FnBodyCallableOp` catches ALL evaluation errors and returns `Ok(Skipped)` for all outputs (line ~258). This hides real evaluator bugs.
- `ExprComputeOp` catches "unknown function" errors and degrades to `Skipped` (line ~790). This is the root cause of T3.
- Missing params → `Value::Skipped` (line ~296).
- Missing ports → `Value::Skipped` (line ~747).
- Pattern/Func callables mark all outputs as optional (line ~1529), so any unwired output silently becomes `Skipped` instead of erroring.

**Fix:** `FnBodyCallableOp` should not catch all errors. If the fn body is invoked with full inputs and evaluation fails, that's a real error. `ExprComputeOp` should not degrade on "unknown function" — unevaluable fn bodies should be detected at compile time (T3's proper fix). Pattern/Func outputs should be required, not optional — the lowerer must wire them or the compilation fails.

### Layer 4 — Driver import resolution (3 sites)

`resolve_import_file_path` in `daglang-driver/src/lib.rs` returns `None` when it can't find an import file. The caller `continue`s, silently dropping the import from the module graph (lines 1639, 1769). A missing import can cause undefined symbols downstream, but the error appears at the use site, not at the import.

**Fix:** Missing import → compile error at the import statement.

### Layer 5 — Codegen/testgen degradation (~15 sites)

- `gunbc_exec::lower().ok()` (line ~4813 in `codegen.rs`) swallows lowering failures and falls back to un-lowered analysis.
- `read_to_string().ok()` / `parse().ok()` (lines 124-125 in `dag_test_discovery.rs`) silently skip modules that can't be read or parsed.
- Corpus identity not found → `continue` after `eprintln!` (line ~5679).
- Effectful node with ExactOutputs → silent DryRun fallback (line ~5705).
- Port lookup failure → default `("String", Cardinality::ONE)` (line ~4902).

**Fix:** Test generation should fail with a clear error when the input DAG can't be lowered or a module can't be parsed. Corpus drift and port lookup failures should be errors, not degraded output.

### Layer 6 — Type system fallbacks (~5 sites)

- Unknown type → `ValueBacking::Json` (line ~786 in `type_registry.rs`).
- Unknown type → `TypeShape::Opaque("Unknown")` (line ~62 in `type_shape.rs`).
- Type resolution error → `None` via `.ok().flatten()` (line ~437).

**Fix:** Unknown types should be a compile error. `ValueBacking::Json` should not be a catchall — every type in the DSL should have an explicit backing.

### Layer 7 — Cache fallbacks (4 sites)

Digest computation, I/O, parsing, and storage errors in `builder.rs` are all silently swallowed (lines 217-253). Caching is documented as "best-effort."

**Fix:** Cache fallbacks are the least severe. Cache miss on error is acceptable behavior — but the error should be logged at debug level so cache corruption is diagnosable. This layer can remain as-is with improved logging.

### Relationship to other items

| Item | Relationship to T13 |
|------|---------------------|
| T3 | Subsumed — resolver-layer fallback (Layer 3) |
| T5 | Subsumed — testgen-layer fallback masking evaluation fallback (Layer 5) |
| T6 | Subsumed — parser-layer lossy fallback (Layer 1) |
| T9 | Subsumed — lowerer-layer silent skip (Layer 2) |
| T10 | Subsumed — lowerer-layer `lower_warn` catalog (Layer 2, partially Layer 5) |
| T12 | Subsumed — display-layer fallback assumptions (Layer 3 / display) |
| T2 | Amplified — `IsString` accepting `Secret` is a type-layer fallback (Layer 6) |
| T7 | Amplified — silent Skipped cascade from missing binding (Layer 3) |
| T11 | Amplified — DryRun passes because fallbacks produce valid-looking output |
| T1 | Independent — executor non-determinism |
| T4 | Independent — conditional merge semantics |
| T8 | Independent — auth architecture |

### Implementation order

1. **Layer 2 first** (lowerer): Delete `lower_warn`, convert all silent `continue` to `Err`. This immediately surfaces every wiring gap at compile time. Most T-items become compile errors instead of runtime mysteries.
2. **Layer 1 next** (parser): Delete `parse_body_lossy`, implement proper error recovery or fail. This eliminates the `lossy: true` codepath and the need for `DeclaredOutputCallableOp`'s optional-output leniency.
3. **Layer 3** (resolver): Remove `Skipped` degradation for eval failures. Make Pattern/Func outputs required.
4. **Layers 4-6** (driver, codegen, types): Convert `None`/`ok()` returns to errors.
5. **Layer 7** (cache): Add debug logging, keep best-effort behavior.
