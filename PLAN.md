# Plan: Compiler Structural Integrity

**Goal:** Eliminate all interpreter fallback, redundant optionality, and
incomplete models from the compiler. When complete, every expression lowers
to structural DAG nodes or fails with a diagnostic. The compiler's Rust
types enforce the DAG's own cardinality system. No silent degradation.

**Drives:** README Invariants 7 (structural lowering) and 8 (construction
not validation). Resolves 13 of 14 POSTMORTEM items and ~65 of ~70 T13
fallback sites.

**Design doc:** `DESIGN_INTERP_FALLBACK.md` contains the full root cause
analysis, fallback site catalog, and impact tracing.

---

## Phase 1: Unified expression lowering (`lower_expr`)

**What:** Transform `resolve_return_expr_source` into a unified
`lower_expr(expr) -> Result<(NodeId, Port), LowerError>` that works for
any expression in any position.

**Steps:**

1. Add 5 context fields to `LoweringContext`:
   - `endpoints_by_full`
   - `uses_binding_types`
   - `active_profile_bindings`
   - `profile_bound_interfaces`
   - `known_interface_types`

2. Add `Expr::Call` arm — look up endpoint, clone callable node, wire
   args, return output. Use existing `clone_loop_body_callable_node`.

3. Add `Expr::ServiceCall` arm — resolve service endpoint, create
   transport triplet, wire args, return parse output. Use existing
   `resolve_service_call_source` and `builder.clone_transport_triplet`.

4. Add `Expr::Pipe` / `Expr::PipeCall` arm — identify collection op via
   `collection_op_kind`, create `LoweredOp::Collection` node, recurse on
   input, wire, return output. Use existing Collection infrastructure.

5. Add `Expr::For` arm — create loop body expansion, wire iterable,
   return output. Use existing loop body infrastructure.

6. Add explicit arms for remaining variants: `Lambda` (error: only valid
   inside pipe), `Map` (structural map construction), `Guarded` (recurse +
   conditional edge), `After` (recurse + ordering edge), `Return` (error:
   handled by `collect_return_bindings`).

7. Delete `_` catch-all. Match is exhaustive.

8. Change return type from `Option<(String, String)>` to
   `Result<(String, String), LowerError>`.

9. Update all 16 call sites to propagate `Result` (the structural
   synthesis functions and `wire_callable_return_outputs`).

10. Remove `is_fn_with_body` skip (line ~8102). All fn/func/pattern items
    go through `lower_expr` + structural return wiring.

11. Delete `synthesize_expr_compute`, `synthesize_tagged_evaluator`,
    `build_evaluator_parts`.

12. Delete `PrimitiveOpKind::ExprCompute`, `PrimitiveOpKind::PipeOp`,
    `PrimitiveOpKind::ForOp`.

13. Delete `ExprComputeOp` and `FnBodyCallableOp` from resolver.

14. Delete `Passthrough` stubs for interpreter-only ops in emitter.

15. Delete `lower_warn`, `DAGLANG_LOWER_WARNINGS` env var.

16. Delete `collect_local_let_bindings` (replaced by `lower_expr`
    recursion through let bindings).

17. Delete `collect_project_fn_bodies` / `sibling_fns` threading
    (runtime evaluator no longer needs fn bodies).

18. Make `lossy_fn_bodies` a compile error (non-empty → fail).

**Acceptance criteria:**

- [ ] `PrimitiveOpKind::ExprCompute` does not exist in the codebase
- [ ] `PrimitiveOpKind::PipeOp` does not exist in the codebase
- [ ] `PrimitiveOpKind::ForOp` does not exist in the codebase
- [ ] `ExprComputeOp` does not exist in the codebase
- [ ] `FnBodyCallableOp` does not exist in the codebase
- [ ] `synthesize_expr_compute` does not exist in the codebase
- [ ] `synthesize_tagged_evaluator` does not exist in the codebase
- [ ] `lower_warn` does not exist in the codebase
- [ ] `DAGLANG_LOWER_WARNINGS` does not exist in the codebase
- [ ] `evaluate_fn_body` is not called from any resolver execution path
- [ ] `resolve_return_expr_source` (now `lower_expr`) returns `Result`,
      not `Option`
- [ ] The match in `lower_expr` is exhaustive — no `_` wildcard
- [ ] Phase 1's statement loop has no `_ => {}` wildcard
- [ ] `HandlerKind::Passthrough` is not used for any `PrimitiveOpKind`
- [ ] No fn/func/pattern item bypasses structural return wiring
- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes

**Resolves:** T3, T5, T9, T10, T12. Partially resolves T6, T13.
Unblocks T11.

---

## Phase 2: Eliminate redundant optionality (all layers)

**What:** Remove all `Option` return types from compiler operations where
the DAG's cardinality system already expresses optionality. Every compiler
operation either succeeds or returns `Result` with a diagnostic.

**Steps:**

1. **Parser (Layer 1):**
   - Delete `parse_body_lossy`. `parse_fn_body`, `parse_func_body`,
     `parse_pattern_body` return `Result<Body, ParseError>` or fail.
   - Fix `consume_brace_block_expr` to parse block expressions properly
     (`let` bindings + final expression) or return `Err`.
   - Delete `lossy: bool` field from body AST nodes.
   - Delete `collect_lossy_fn_bodies` and `CompileOutput.lossy_fn_bodies`.

2. **Driver (Layer 4):**
   - Change `resolve_import_file_path` from `Option<PathBuf>` to
     `Result<PathBuf, ImportError>`.
   - All 3 call sites propagate the error.

3. **Codegen (Layer 5):**
   - Replace `gunbc_exec::lower().ok()` with `?` propagation.
   - Replace `read_to_string().ok()` and `parse().ok()` with `?`.
   - Replace corpus identity `None => continue` with `Err`.
   - Replace effectful node DryRun fallback with `Err`.
   - Replace port lookup `unwrap_or(("String", ONE))` with `Err`.
   - Replace `probe_best_response` fallback with `Err`.

4. **Types (Layer 6):**
   - Change `value_backing_for_type_id` to return `Result`. Unknown
     types are compile errors, not `ValueBacking::Json`.
   - Change `resolve_type` to propagate errors instead of
     `.ok().flatten()`.
   - Remove `TypeShape::Opaque("Unknown")` — unknown shapes are errors.

5. **Cache (Layer 7):**
   - Add debug logging for digest, I/O, parse, and store errors.
   - Keep `Option` — cache is intentionally best-effort.

**Acceptance criteria:**

- [ ] `parse_body_lossy` does not exist in the codebase
- [ ] `consume_brace_block_expr` returns `Result`, not empty `Expr::Record`
- [ ] `lossy` field does not exist on body AST nodes
- [ ] `resolve_import_file_path` returns `Result`, not `Option`
- [ ] No `.ok()` calls that swallow `Result` in codegen/testgen
      (excluding cache layer)
- [ ] `ValueBacking::Json` is not a catchall for unknown types
- [ ] `TypeShape::Opaque("Unknown")` does not exist
- [ ] No `.ok().flatten()` in type resolution
- [ ] Cache errors are logged at debug level
- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes

**Resolves:** T6 (fully). Substantially addresses T13 (Layers 1, 4, 5, 6
— ~15 additional sites).

---

## Phase 3: Complete incomplete models

**What:** Finish the half-built extensions in the IR, type system, and
resource model. Each item has an existing design — the implementation was
never connected.

### 3a: Edge ordering (T1)

- In the lowerer, assign monotonic `Edge.index` values when creating
  fan-in edges (multiple edges to the same list port).
- In the executor, sort fan-in values by `edge.index` before constructing
  `Value::List`.
- Revert the lenient `a.len() == b.len()` comparison in `window.rs` to
  exact equality.

**Acceptance criteria:**

- [ ] Fan-in `Value::List` ordering is deterministic across runs
- [ ] `Edge.index` is non-zero for fan-in edges
- [ ] Window test assertions use exact value equality

### 3b: Secret as first-class ValueBacking (T2)

- Add `ValueBacking::Secret` variant.
- Register it in `TypeRegistry` for `TypeId("Secret")`.
- Add `OutputMatcher::IsSecret` variant.
- Auto-mock emits `IsSecret` for Secret-typed ports.
- Remove `IsString` accepting `Value::Secret` workaround.

**Acceptance criteria:**

- [ ] `ValueBacking::Secret` exists
- [ ] `OutputMatcher::IsString` does not accept `Value::Secret`
- [ ] Secret-typed ports produce `IsSecret` matchers

### 3c: ConditionalMerge node (T4)

- Add `ConditionalMerge` node kind to IR.
- Lowerer emits `ConditionalMerge` at branch join points.
- Executor has explicit semantics: exactly one non-Skipped input expected,
  error on zero or 2+.
- Remove ad-hoc scalar fan-in overwrite logic from executor.

**Acceptance criteria:**

- [ ] `ConditionalMerge` node kind exists in IR
- [ ] Executor does not have ad-hoc scalar fan-in overwrite logic
- [ ] Conditional branches produce deterministic, validated merge

### 3d: Filesystem transport binding (T7)

- Implement concrete Filesystem transport (file read/write/probe via
  real I/O).
- `InterfaceStubExecuteOp` is not used for Filesystem operations.
- `std.patterns.read_text_files` and `classify_files` work in Real mode.

**Acceptance criteria:**

- [ ] Filesystem `probe`, `read`, `write` work in Real mode
- [ ] `InterfaceStubExecuteOp` is not reached for Filesystem operations

### 3e: Credential provider interface (T8)

- Define `CredentialProvider` interface with `resolve(scope) -> Secret`.
- Concrete bindings: `EnvVarProvider`, `GcpSecretManagerProvider`.
- Services declare credential requirements as capabilities, not explicit
  input fields.
- Credential resolution dispatched by execution profile.

**Acceptance criteria:**

- [ ] `github_token()` in `auth.dag` does not hand-roll credential
      materialization
- [ ] Credential resolution uses the interface + binding + profile pattern
- [ ] Adding a new service with auth needs does not duplicate
      materialization logic

---

## Phase 4: Migration and test coverage

### 4a: Hardcoded paths → WorkspaceLayout (T14)

- Extend `WorkspaceLayout` with `generated_tests_src_dir()` and core
  source root accessors.
- Migrate all `"src/core"`, `"src/generated-tests/src"`, and
  `"../../core/ir"` literals to use `WorkspaceLayout`.
- Extend `dsl/config/codegen_paths.dag` with generated-tests root.

**Acceptance criteria:**

- [ ] No `"src/core"` string literals in production path construction
- [ ] No `"../../core/"` relative path fallbacks in production code
- [ ] All paths derived from `WorkspaceLayout` or `codegen_paths.dag`

### 4b: Tiered test execution (T11)

- Testgen identifies pure subgraphs (no external I/O) and generates
  Real-mode tests for them.
- Env var reads use `std::env::set_var` in tests (zero I/O).
- Filesystem operations use tempdir/VFS in tests.
- Transport-execute nodes remain DryRun-mocked.

**Acceptance criteria:**

- [ ] At least one generated test suite runs in Real mode for pure
      subgraphs
- [ ] Credential flow has a Real-mode integration test (sets
      `GITHUB_TOKEN` env var, verifies non-Skipped `Value::Secret`)
- [ ] T11's incident table: every row either has a Real-mode test or is
      documented as DryRun-only with justification

---

## Final State

When all four phases are complete:

### Deleted from codebase

| Component | Reason |
|---|---|
| `PrimitiveOpKind::ExprCompute` | Replaced by structural lowering |
| `PrimitiveOpKind::PipeOp` | Pipes lower to `LoweredOp::Collection` |
| `PrimitiveOpKind::ForOp` | For-loops lower to loop body expansion |
| `ExprComputeOp` | No interpreter-backed nodes in DAG |
| `FnBodyCallableOp` | All fn items use structural return wiring |
| `synthesize_expr_compute` | `lower_expr` handles all expressions |
| `synthesize_tagged_evaluator` | `lower_expr` handles all expressions |
| `build_evaluator_parts` | No evaluator parts needed |
| `parse_body_lossy` | Parser succeeds or fails |
| `consume_brace_block_expr` (lossy) | Parses block expressions properly |
| `lower_warn` | `lower_expr` returns `Result` |
| `DAGLANG_LOWER_WARNINGS` | No warnings — errors or success |
| `collect_local_let_bindings` | `lower_expr` recurses through lets |
| `collect_project_fn_bodies` | Runtime evaluator not used |
| `lossy_fn_bodies` field | Lossy bodies are compile errors |
| `ValueBacking::Json` as catchall | Unknown types are compile errors |
| `TypeShape::Opaque("Unknown")` | Unknown shapes are compile errors |
| `HandlerKind::Passthrough` for interp ops | No interpreter-only ops |
| `InterfaceStubExecuteOp` for Filesystem | Concrete binding exists |
| Ad-hoc scalar fan-in overwrite | `ConditionalMerge` node |
| `IsString` accepting `Value::Secret` | `IsSecret` matcher |
| Hand-rolled credential functions | Credential provider interface |
| Hardcoded `"src/core"` path literals | `WorkspaceLayout` derivation |

### Structural guarantees (enforced by Rust type system)

| Guarantee | Mechanism |
|---|---|
| Every `Expr` variant has a lowering | Exhaustive match in `lower_expr`, no `_` wildcard |
| New `Expr` variant → compile error until handled | Rust exhaustive match |
| Lowering succeeds or produces diagnostic | `Result` return type |
| No silent degradation in lowerer | `lower_warn` deleted, `Result` propagated |
| No silent degradation in parser | `parse_body_lossy` deleted |
| No silent degradation in driver | `resolve_import_file_path` returns `Result` |
| No silent degradation in codegen | No `.ok()` swallowing errors |
| No silent degradation in types | No catchall defaults |
| Optionality expressed only in DAG cardinality | No compiler-level `Option` for fallible ops |
| Fan-in ordering is deterministic | `Edge.index` populated, executor sorts |
| Conditional merge is explicit in IR | `ConditionalMerge` node |
| Secret is a first-class type | `ValueBacking::Secret` |

### POSTMORTEM resolution

| Item | Status |
|---|---|
| T1 (fan-in ordering) | Resolved (Phase 3a) |
| T2 (Secret backing) | Resolved (Phase 3b) |
| T3 (unevaluable fn bodies) | Resolved (Phase 1) |
| T4 (conditional merge) | Resolved (Phase 3c) |
| T5 (literal source skip) | Resolved (Phase 1) |
| T6 (parser lossy fallback) | Resolved (Phase 2) |
| T7 (Filesystem binding) | Resolved (Phase 3d) |
| T8 (auth architecture) | Resolved (Phase 3e) |
| T9 (local variable wiring) | Resolved (Phase 1) |
| T10 (lower_warn diagnostics) | Resolved (Phase 1) |
| T11 (DryRun-only tests) | Resolved (Phase 4b) |
| T12 (silent Skipped cascade) | Resolved (Phase 1) |
| T13 (zero-fallback policy) | Resolved (~65/70 sites; ~5 cache sites documented as intentional) |
| T14 (hardcoded paths) | Resolved (Phase 4a) |

### T13 fallback site accounting

| Layer | Sites | Disposition |
|---|---|---|
| 1 (parser) | 8 | Eliminated (Phase 2) |
| 2 (lowerer) | ~30 | Eliminated (Phase 1) |
| 3 (resolver) | ~10 | Eliminated (Phase 1) |
| 4 (driver) | 3 | Eliminated (Phase 2) |
| 5 (codegen) | ~15 | Eliminated (Phase 2) |
| 6 (types) | ~5 | Eliminated (Phase 2) |
| 7 (cache) | ~4 | Documented as intentional best-effort |
| **Total** | **~70** | **~65 eliminated, ~5 accepted** |
