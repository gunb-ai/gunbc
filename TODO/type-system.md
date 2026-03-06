# Lane 1: Type System

**Design doc**: [`docs/design/v4/compositional-type-coverage.md`](../docs/design/v4/compositional-type-coverage.md)
**Principle**: Decisions obligate. Obligations propagate. Propagation is automatic.
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`

> **Lesson**: Design the full layer stack once, implement once. Transport/service went through 4 waves
> because each wave discovered one more layer was needed. For each workstream here: understand the
> final state before writing code. No intermediate abstractions that get replaced next week.

---

## Status

```
WS-1: std/ Primitive Vocabulary    — 7/8 complete
WS-2: Service Type Discipline      — 0/5 complete  (blocked by WS-1)
WS-3: Typechecker Unification      — 0/8 complete
WS-4: Presence Axis                — 0/5 complete  (blocked by WS-3)
WS-5: Type DAG Execution           — 0/4 complete  (blocked by WS-3)
WS-6: Tool/Workflow Completeness   — 0/5 complete  (blocked by WS-1, WS-2)
WS-7: Extern Linking               — 4/5 complete  (NF-1:6 landed most of this)
```

### Dependency Graph

```
WS-1 (std/ cleanup)  ──→  WS-2 (service discipline)  ──→  WS-6 (tool/workflow)
                                                              ↑
WS-3 (typechecker)   ──→  WS-4 (presence axis)        WS-7 (extern linking)
                      ──→  WS-5 (type DAG execution)
```

WS-1 and WS-3 can start immediately. WS-7 is mostly done.

---

## WS-1: std/ Primitive Vocabulary

**Goal**: Make `std/` a reference-quality foundation that downstream layers build on.
**Design ref**: `compositional-type-coverage.md` § WS-1, § "std/ Primitives"
**Success criteria**: Zero `String` fields where a refinement type exists. Zero duplicates. Zero stubs without `@testgen_skip` or deletion.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS1-1 | **Timestamp consistency.** Replace ~20 `String` timestamp fields with `Timestamp` across `types.dag`, `resources.dag`. | `grep -r 'created_at: String\|updated_at: String\|timestamp: String' dsl/std/` returns 0 hits. | S | **Done** |
| 2 | WS1-2 | **Enum extraction.** Convert ~15 stringly-typed enumerations to sum types: `TopologyNodeKind`, `DocSourceKind`, `SeverityLevel`, `DataSource`, `RetryTrigger`, etc. | Each converted type is a `type X = A \| B \| C` sum type. `grep 'kind: String' dsl/std/` returns 0 hits for known closed sets. | M | **Done** |
| 3 | WS1-3 | **Brand application.** Apply `ContentHash` brand to `StageRunKey.input_hash`, `Artifact.content_hash`, `ArtifactMarker.content_hash`. | Fields reference `ContentHash` type (not bare `String`). Verify in `dsl/std/types.dag`. | S | **Done** |
| 4 | WS1-4 | **Duration unit types.** Create `Seconds` and `Milliseconds` branded types. Apply consistently. | `type Seconds = Int @brand("Seconds")` and `type Milliseconds = Int @brand("Milliseconds")` exist in `dsl/std/`. All duration fields use one or the other. | S | **Done** |
| 5 | WS1-5 | **Duplicate resolution.** Merge two `RetryPolicy` definitions. Deduplicate `EntryKind`/`SymlinkTarget`. | `grep -c 'type RetryPolicy' dsl/` returns 1. `grep -c 'type EntryKind' dsl/` returns 1. | S | **Done** |
| 6 | WS1-6 | **Missing types.** Add: `SeverityLevel`, `DataSource`, `RetryTrigger`, `LanguageId`, `GcpRegion`, canonical error wrapper, C/MIPS/Dag language definitions. | Each type exists as a sum type or branded alias in `dsl/std/`. No stringly-typed references to these concepts elsewhere. | M | **Done** (`LanguageId` at `std/types.dag:82`, `GcpRegion` at `extdeps/cloud/gcp/core.dag:88`) |
| 7 | WS1-7 | **Stub cleanup.** Address 8 stubs that look like features. Each must be implemented, deleted, or marked `@testgen_skip`. | `grep -r '@testgen_skip' dsl/std/` accounts for all remaining stubs. Zero unmarked stubs. | M | Open |
| 8 | WS1-8 | **`Filesystem.read` type fix.** Change `path: String` to `path: TextFilePath`. | `dsl/std/resources.dag` Filesystem.read signature uses `TextFilePath`. | S | **Done** |

---

## WS-2: Service Layer Type Discipline

**Goal**: Services use the types their extdeps define.
**Design ref**: `compositional-type-coverage.md` § WS-2, § "services/ — Type Discipline Gap"
**Blocked by**: WS-1
**Success criteria**: Zero dead imports. Zero `Json` escape hatches. Every GET declares `readonly`. Every BearerToken service has `auth_input`.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS2-1 | **Dead import audit.** Use or delete every import in each service file. | Every `import` in `dsl/services/**/*.dag` has at least one reference in the file body. | S | Open |
| 2 | WS2-2 | **Input/output type upgrades.** Replace `String`/`Json` with domain types already imported. | `grep 'input:.*Json\|output:.*Json' dsl/services/` returns 0 for operations that have typed extdep shapes. | L | Open |
| 3 | WS2-3 | **Behavioral property completion.** Add `readonly` to GET/list ops, `idempotent` to mutations. | Every `method: GET` or `method: LIST` operation has `readonly` keyword. Every PUT/DELETE has `idempotent`. | S | Open |
| 4 | WS2-4 | **`auth_input` completion.** Add to `issues.dag`, `pull_request.dag`, `anthropic.dag`, `openai.dag`. | `grep -L 'auth_input' dsl/services/github/*.dag dsl/services/llm/*.dag` returns 0 files (all have it). | S | Open |
| 5 | WS2-5 | **`owner`/`repo` as service config params.** Formalize service-level path parameters. | `owner` and `repo` declared once in service `config {}` block, not repeated per-operation. | M | Open |

---

## WS-3: Unify DSL Typechecker with IR TypeRegistry

**Goal**: One type world. Compatibility walks type DAG per-layer, comparing explicit node contracts.
**Design ref**: `compositional-type-coverage.md` § WS-3, § Gap 2, § Gap 3
**No blockers** — can start immediately.

> **Lesson**: This is the highest-risk workstream for a→b→...→f drift. WS3-1 through WS3-4 are
> deeply coupled — changing the type representation (WS3-1/WS3-2) without changing the checker
> (WS3-3) creates a half-migrated state. Design WS3-1:4 as one unit before implementing any of them.
> The `normalize_type_id` deletion (WS3-3) is the proof that the new system works.

**Success criteria**: `normalize_type_id` deleted. Every `TypeOp` carries explicit node contract. All checks walk type DAGs per-layer. Exhaustiveness is static. `readonly`/`idempotent` declarations validated against call graph. `OperationBehavior` consumed for test/retry generation.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS3-1 | **Explicit node contracts on `TypeOp`.** Each variant declares its effect on all three dimensions (base type, cardinality, predicates) as `Set`/`Add`/`Inherited`. Replaces `TypeContract` reverse-engineering. | Every `TypeOp` variant has a `fn contract(&self) -> NodeContract` method. `TypeContract` struct deleted from `core/ir/`. | L | Open |
| 2 | WS3-2 | **DSL type definitions → `Dag<TypeOp>` at parse time.** Each type becomes a layered DAG with explicit node contracts. | `daglang-typecheck` produces `Dag<TypeOp>` for every type definition. Old string-based type representation unused. | XL | Open |
| 3 | WS3-3 | **Typechecker uses per-layer node contract comparison.** Replace string-based `normalize_type_id` with per-layer DAG walk comparing node contracts. | `normalize_type_id` function **deleted**. `grep -r 'normalize_type_id' core/` returns 0. All compatibility checks use DAG walk. | L | Open |
| 4 | WS3-4 | **Optionality is a DAG layer.** `T?` → `Wrap(Optional)` with cardinality `Set([0,1])`, not string suffix. | `Port::is_optional()` no longer calls `ends_with('?')`. `grep "ends_with.*?" core/ir/src/` returns 0 for optionality checks. Cardinality mismatch is a type error. | L | Open |
| 5 | WS3-5 | **Branch type unification.** `if/else` and `match` arms compute `join` (LUB) of type DAGs per-layer. | Test: `if cond { "hello" } else { 42 }` produces type error. Matching arms with different types errors. | M | Open |
| 6 | WS3-6 | **Match exhaustiveness.** `Coproduct` variants checked statically from type DAG. | Test: `match x { A => ... }` on `type X = A \| B` is a compile error. Non-exhaustive match on known sum type = error. | M | Open |
| 7 | WS3-7 | **Behavioral property enforcement (Level 2).** Validate `readonly`/`idempotent` declarations against `CallableProperties` BFS results (`daglang-derive`). | Test: `func snapshot() readonly { call_write_service() }` = compile error `E5001`. Validation pass exists in `daglang-typecheck` or `daglang-derive`. | M | Open |
| 8 | WS3-8 | **Behavioral contract consumption (Level 3).** Compiler reads `OperationBehavior` from extdeps (`idempotency_keys`, `determinism`, `failure_modes`). | `OperationBehavior` fields consumed by at least one compiler pass. `idempotency_keys` checked at retry sites (`E5003`). Test: retry on non-idempotent operation = warning/error. | L | Open |

---

## WS-4: Presence Axis on Ports

**Goal**: Guard-skippable outputs cannot silently feed required inputs. I2.5 implemented.
**Design ref**: `compositional-type-coverage.md` § WS-4, § Gap 1
**Blocked by**: WS-3
**Success criteria**: Zero silent `Skipped → concrete_value` coercions. `Value::Skipped` unreachable on `Required` ports. Every fallback explicit.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS4-1 | **Add `presence: PresenceMode` to `Port`.** `Required \| Guardable`. Guards produce `Guardable` on outputs. | `Port` struct in `core/ir/src/` has `presence: PresenceMode` field. Guard node outputs tagged `Guardable`. | M | Open |
| 2 | WS4-2 | **`DagBuilder::add_edge` rejects `Guardable → Required`.** Without explicit narrowing node. | Test: connecting guard output to required input = `DagBuilderError`. | M | Open |
| 3 | WS4-3 | **Add `default`/`require` narrowing operators.** DAG-level nodes that convert `Guardable` to `Required`. | `NarrowingOp::Default(fallback_value)` and `NarrowingOp::Require` exist in `core/ir/src/patterns/`. | M | Open |
| 4 | WS4-4 | **Eliminate 7 silent Skipped coercion sites.** Convert to errors. | `grep -r 'Value::Skipped =>' core/exec/` shows only explicit error handling, no silent conversion to concrete values. | L | Open |
| 5 | WS4-5 | **Eliminate evaluator silent behaviors.** 12 operations that silently default instead of erroring. | `grep -r 'unwrap_or\|unwrap_or_default\|unwrap_or_else' core/exec/src/` shows zero silent defaults for Skipped inputs. | L | Open |

---

## WS-5: Type DAG Execution (The Full Vision)

**Goal**: Type DAG per-layer operations become actual workflow nodes. Coercion = visible graph nodes.
**Design ref**: `compositional-type-coverage.md` § WS-5
**Blocked by**: WS-3
**Success criteria**: Every coercion is a visible node. Every downcast has validation nodes. `TypeShape::Opaque` trends to zero.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS5-1 | **Coercion insertion at lower time.** Lowerer inserts nodes that add/remove type DAG layers. | `daglang-lower` produces explicit coercion nodes (visible in `daglang viz` output). No implicit coercions at executor time. | XL | Open |
| 2 | WS5-2 | **Downcast validation nodes.** `as Url` inserts per-layer validation nodes. | `as` expressions lower to validation DAG nodes. Test: `"not a url" as Url` fails at runtime via validation node. | L | Open |
| 3 | WS5-3 | **Witness-driven test generation.** Each layer's constraints generate boundary test cases. | Testgen produces boundary tests from type constraints (e.g., `@range(min: 0, max: 100)` generates tests for -1, 0, 100, 101). | L | Open |
| 4 | WS5-4 | **TypeShape consumed by emitters.** Replace string matching with `TypeShape` dispatch. | `grep -r 'TypeShape::Opaque' core/daglang/daglang-emit/` returns 0. All emission uses specific `TypeShape` variants. | M | Open |

---

## WS-6: Tool/Workflow Completeness

**Goal**: All .dag files compile and have real bodies.
**Design ref**: `compositional-type-coverage.md` § WS-6, § "tools/ and workflows/"
**Blocked by**: WS-1, WS-2
**Success criteria**: All `.dag` files compile. Zero empty stages. Every `func` with resource use declares `uses`.

> **Lesson**: Prove compilation before building on top. SDLC had elaborate infrastructure built on
> .dag files that never compiled. For WS6-4 and WS6-5: each stage body must compile and pass a
> test before moving to the next stage.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS6-1 | **Fix `testgen.dag`.** Declare missing `generate` as extern func, or implement. | `build_dsl_graph("tools.testgen")` succeeds in a Rust test. | S | Open |
| 2 | WS6-2 | **Fix `deps.dag`.** Declare missing externs or implement. | `build_dsl_graph("tools.deps")` succeeds in a Rust test. | S | Open |
| 3 | WS6-3 | **Add `uses` declarations.** `makegen`, `pragma`, `build` funcs. | Every `func` that calls file/shell transport has a `uses` declaration. `daglang check dsl/tools/` reports 0 missing `uses` warnings. | S | Open |
| 4 | WS6-4 | **Fill `ci.dag` stage bodies.** Wire 12 stages to tool funcs. | `grep -c 'stage.*{[[:space:]]*}' dsl/pipelines/ci.dag` returns 0. Each stage has a non-empty body. Compilation test passes. | L | Open |
| 5 | WS6-5 | **Fill remaining workflow stage bodies.** `gist.dag`, `pragma.dag`, others. | `grep -rn 'stage.*{[[:space:]]*}' dsl/pipelines/` returns 0 across all pipeline files. | L | Open |

---

## WS-7: Extern Linking

**Goal**: Missing extern symbol = hard error. No fallbacks.
**Design ref**: `compositional-type-coverage.md` § WS-7
**No blockers** — independent.

> **Already done (NF-1:6)**: `extern func`/`extern asset` parse and lower. `ExternResolver` trait
> exists (simpler than the designed `Backend` — this is fine). `CompileReceipt` with 3-part digest
> works. `EMBED_REGISTRY`, `is_makegen_module()`, `UnimplementedPassthrough` all deleted.
> Remaining work: migrate the 10 extern impls in `gunbc-app/src/extern_ops.rs` to DSL.

**Success criteria**: `extern_ops.rs` deleted or contains only operations that provably require Rust (recursive types, inventory access). Zero passthrough fallbacks. Deterministic receipts (already done).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS7-1 | **Phase A: Introduce externs.** `extern func`/`extern asset` in parser/typechecker/lowering. | Extern declarations parse, typecheck, and lower. 5 tests pass. | L | **Done** (NF-1, 2026-02-25) |
| 2 | WS7-2 | **Phase B: Compile-time resolution with hard errors.** `ExternResolver` trait + `resolve_extern_call()` hard-fails on unresolved symbols. | Unresolved extern = compile error with diagnostic. `ExternResolver` trait in `core/resolve/src/lib.rs`. | L | **Done** (NF-2:4, 2026-02-25) |
| 3 | WS7-3 | **Phase C: Migrate remaining extern impls to DSL.** 10 impls in `extern_ops.rs` (521 LOC). Each must be either migrated to DSL evaluation or justified as Rust-only. | `extern_ops.rs` line count < 200. Each remaining impl has a comment explaining why DSL can't handle it (e.g., recursive types, inventory). Migrated items: **deleted** from extern_ops.rs, body lives in `.dag` file evaluated by `evaluate_fn_body()`. | L | Open |
| 4 | WS7-4 | **Phase D: Remove fallback surfaces.** Delete `EMBED_REGISTRY`, `is_makegen_module()`, `UnimplementedPassthrough`, etc. | Zero fallback surfaces remain. | M | **Done** (NF-5, 2026-02-25) |
| 5 | WS7-5 | **Phase E: Determinism hardening.** `CompileReceipt` hashes, CI gates, diagnostic ordering. | `CompileReceipt` exists with 3-part digest. `normalize_diagnostics()` enforces ordering. 4 determinism tests pass. | M | **Done** (NF-6, 2026-02-25) |

---

## Backlog

### Compiler Extensibility

| ID | What | Redundant / Total LOC | Size | Notes |
|----|------|:---------------------:|------|-------|
| CX-1 | **Pipe Method Registry.** Single `PipeMethodDef` table replaces metadata boilerplate across 10 sites. | 514 / 1,010 | M | Prerequisite for FC-CF2/CF3. |
| CX-2 | **Deduplicate `lower_expr`/`remap_expr_idents`.** **Latent bug**: variant constructors silently misclassified. | 220 / 297 | S | Fix latent bug + delete ~148 lines. |
| CX-3 | **Type mapping consolidation.** 3 Go/Rust type sites bypass `DslTypeMapping` table. | 36 / 68 | S | Low priority. |
| CX-4 | **Callable Item group helper.** `CallableItem` trait for FuncDef/PatternDef. | 70 / 304 | S | Won't touch typecheck main validation. |
| CX-5 | **Structural primitive `is_structural()`.** 11 variants × 3 sites, 100% redundant. | 60 / 60 | S | Single method. |

### Compiler Features

| ID | Feature | Size | Notes |
|----|---------|------|-------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | After CX-1 |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | After CX-1 |

### Observations

| # | Smell | Observation | File |
|---|-------|-------------|------|
| 1 | Static mapping table | `default_rest_response()` grows per service type | `core/test/src/auto_mock.rs` |
