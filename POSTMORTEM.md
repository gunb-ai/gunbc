# POSTMORTEM

Rolling postmortem — technical debt and incidents identified during development.
Each item has enough context to be picked up cold.

**Origin:** T1–T8 identified during the generated test fix-up (2026-03-08).
T9–T11 identified during the `make gist` credential failure investigation (2026-03-08).

**Policy:** These items are interconnected. Do not spot-fix individual items —
they need to be considered holistically and designed together before any
implementation begins. Many share root causes (e.g., T6/T9/T10 are facets of
the same lowering gap; T7/T8 are facets of the same auth architecture gap;
T1/T4/T11 are facets of the same executor semantics gap). A spot fix in one
area will shift the failure to another. The goal is a single cohesive design
pass that resolves the underlying structural issues.

**Clusters:**

- **Lowering fidelity** (T6, T9, T10): Lossy parsing → broken wiring → silent Skipped outputs → invisible via warnings-off-by-default. One design: fix the parser (T6), which eliminates T9; then promote all warnings to errors (T10) so future gaps are caught at compile time.
- **Auth & resource architecture** (T7, T8): No concrete Filesystem binding + hand-rolled credential materialization. One design: credential provider interface (T8) with concrete transport bindings for Filesystem (T7), resolved via execution profiles.
- **Executor semantics** (T1, T4): Non-deterministic fan-in + ad-hoc conditional merge. One design: deterministic edge ordering (T1) + formal ConditionalMerge node (T4), making execution order explicit in the IR.
- **Type system gaps** (T2, T3, T5): Secret not first-class + unevaluable fn bodies + literal source evaluation. One design: complete the ValueBacking model (T2), add evaluability analysis (T3), and use it for smarter example generation (T5).
- **Test coverage model** (T11): DryRun-only tests hide all of the above. One design: tiered execution with selective mocking, virtual backends for testable I/O, Real-mode tests for pure subgraphs.

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
