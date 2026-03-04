# Lane 1: Type System

**Design doc**: [`docs/design/v4/compositional-type-coverage.md`](../docs/design/v4/compositional-type-coverage.md)
**Principle**: Decisions obligate. Obligations propagate. Propagation is automatic.
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`

---

## Status

```
WS-1: std/ Primitive Vocabulary    — 0/8 complete
WS-2: Service Type Discipline      — 0/5 complete  (blocked by WS-1)
WS-3: Typechecker Unification      — 0/8 complete
WS-4: Presence Axis                — 0/5 complete  (blocked by WS-3)
WS-5: Type DAG Execution           — 0/4 complete  (blocked by WS-3)
WS-6: Tool/Workflow Completeness   — 0/5 complete  (blocked by WS-1, WS-2)
WS-7: Extern Linking               — 0/5 complete
```

### Dependency Graph

```
WS-1 (std/ cleanup)  ──→  WS-2 (service discipline)  ──→  WS-6 (tool/workflow)
                                                              ↑
WS-3 (typechecker)   ──→  WS-4 (presence axis)        WS-7 (extern linking)
                      ──→  WS-5 (type DAG execution)
```

WS-1 and WS-3 can start immediately. WS-7 is independent.

---

## WS-1: std/ Primitive Vocabulary

**Goal**: Make `std/` a reference-quality foundation that downstream layers build on.
**Design ref**: `compositional-type-coverage.md` § WS-1, § "std/ Primitives"
**Success criteria**: Zero `String` fields where a refinement type exists. Zero duplicates. Zero stubs without `@testgen_skip` or deletion.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS1-1 | **Timestamp consistency.** Replace ~20 `String` timestamp fields with `Timestamp` across `types.dag`, `resources.dag`. | Zero `String` fields where `Timestamp` is the semantic intent. | S | Open |
| 2 | WS1-2 | **Enum extraction.** Convert ~15 stringly-typed enumerations to sum types: `TopologyNodeKind`, `DocSourceKind`, `SeverityLevel`, `DataSource`, `RetryTrigger`, etc. | Zero `String` fields with known closed value sets. | M | Open |
| 3 | WS1-3 | **Brand application.** Apply `ContentHash` brand to `StageRunKey.input_hash`, `Artifact.content_hash`, `ArtifactMarker.content_hash`. | All content hash fields use branded `ContentHash` type. | S | Open |
| 4 | WS1-4 | **Duration unit types.** Create `Seconds` and `Milliseconds` branded types. Apply consistently. | Unit-branded types distinguish seconds from milliseconds. | S | Open |
| 5 | WS1-5 | **Duplicate resolution.** Merge two `RetryPolicy` definitions. Deduplicate `EntryKind`/`SymlinkTarget`. | Zero duplicate type definitions across std/. | S | Open |
| 6 | WS1-6 | **Missing types.** Add: `SeverityLevel`, `DataSource`, `RetryTrigger`, `LanguageId`, `GcpRegion`, canonical error wrapper, C/MIPS/Dag language definitions. | Every semantic concept has one authoritative type. | M | Open |
| 7 | WS1-7 | **Stub cleanup.** Address 8 stubs that look like features. Each must be implemented, deleted, or marked `@testgen_skip`. | Zero stubs that look like features. | M | Open |
| 8 | WS1-8 | **`Filesystem.read` type fix.** Change `path: String` to `path: TextFilePath`. | Comment matches signature; refined type used. | S | Open |

---

## WS-2: Service Layer Type Discipline

**Goal**: Services use the types their extdeps define.
**Design ref**: `compositional-type-coverage.md` § WS-2, § "services/ — Type Discipline Gap"
**Blocked by**: WS-1
**Success criteria**: Zero dead imports. Zero `Json` escape hatches. Every GET declares `readonly`. Every BearerToken service has `auth_input`.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS2-1 | **Dead import audit.** Use or delete every import in each service file. | Zero dead imports across all service .dag files. | S | Open |
| 2 | WS2-2 | **Input/output type upgrades.** Replace `String`/`Json` with domain types already imported. | Zero `Json` escape hatches where typed shapes exist. | L | Open |
| 3 | WS2-3 | **Behavioral property completion.** Add `readonly` to GET/list ops, `idempotent` to mutations. | Every GET declares `readonly`. Every idempotent op declared. | S | Open |
| 4 | WS2-4 | **`auth_input` completion.** Add to `issues.dag`, `pull_request.dag`, `anthropic.dag`, `openai.dag`. | Every BearerToken service has `auth_input`. | S | Open |
| 5 | WS2-5 | **`owner`/`repo` as service config params.** Formalize service-level path parameters. | Path parameters declared once at service level. | M | Open |

---

## WS-3: Unify DSL Typechecker with IR TypeRegistry

**Goal**: One type world. Compatibility walks type DAG per-layer, comparing explicit node contracts.
**Design ref**: `compositional-type-coverage.md` § WS-3, § Gap 2, § Gap 3
**No blockers** — can start immediately.
**Success criteria**: `normalize_type_id` deleted. Every `TypeOp` carries explicit node contract. All checks walk type DAGs per-layer. Exhaustiveness is static. `readonly`/`idempotent` declarations validated against call graph. `OperationBehavior` consumed for test/retry generation.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS3-1 | **Explicit node contracts on `TypeOp`.** Each variant declares its effect on all three dimensions (base type, cardinality, predicates) as `Set`/`Add`/`Inherited`. Replaces `TypeContract` reverse-engineering. New node types must declare all three — fail-closed. | Every `TypeOp` variant carries explicit three-dimension contract. | L | Open |
| 2 | WS3-2 | **DSL type definitions → `Dag<TypeOp>` at parse time.** Each type becomes a layered DAG with explicit node contracts. | Every DSL type definition produces a `Dag<TypeOp>`. | XL | Open |
| 3 | WS3-3 | **Typechecker uses per-layer node contract comparison.** Replace string-based `normalize_type_id` with per-layer DAG walk comparing node contracts. | `normalize_type_id` deleted. All compatibility through per-layer walk. | L | Open |
| 4 | WS3-4 | **Optionality is a DAG layer.** `T?` → `Wrap(Optional)` with cardinality `Set([0,1])`, not string suffix. `Port::is_optional()` no longer uses `ends_with('?')`. | `T` and `T?` not interchangeable. Cardinality mismatch is type error. | L | Open |
| 5 | WS3-5 | **Branch type unification.** `if/else` and `match` arms compute `join` (LUB) of type DAGs per-layer. | Branch arms produce unified types. Mismatch is type error. | M | Open |
| 6 | WS3-6 | **Match exhaustiveness.** `Coproduct` variants checked statically from type DAG. | Non-exhaustive match on known sum type is compile error. | M | Open |
| 7 | WS3-7 | **Behavioral property enforcement (Level 2).** Validate `readonly`/`idempotent` declarations against `CallableProperties` BFS results. `func snapshot() readonly` that calls non-readonly service = compile error `E5001`. Infrastructure exists (`daglang-derive` BFS); validation pass missing. | Declared behavioral properties validated against derived properties. Contradiction = compile error. | M | Open |
| 8 | WS3-8 | **Behavioral contract consumption (Level 3).** Compiler reads `OperationBehavior` from extdeps (`idempotency_keys`, `determinism`, `failure_modes`). Generates: retry policy constraints from `idempotency_keys`, test assertion constraints from `determinism`, error classifier hints from `failure_modes`. | `OperationBehavior` data consumed by compiler. At minimum: `idempotency_keys` checked at retry sites (`E5003`). | L | Open |

---

## WS-4: Presence Axis on Ports

**Goal**: Guard-skippable outputs cannot silently feed required inputs. I2.5 implemented.
**Design ref**: `compositional-type-coverage.md` § WS-4, § Gap 1
**Blocked by**: WS-3
**Success criteria**: Zero silent `Skipped → concrete_value` coercions. `Value::Skipped` unreachable on `Required` ports. Every fallback explicit.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS4-1 | **Add `presence: PresenceMode` to `Port`.** `Required \| Guardable`. Guards produce `Guardable` on outputs. | Port struct has presence field. Guard outputs are `Guardable`. | M | Open |
| 2 | WS4-2 | **`DagBuilder::add_edge` rejects `Guardable → Required`.** Without explicit narrowing node. | Builder error on `Guardable → Required` without narrowing. | M | Open |
| 3 | WS4-3 | **Add `default`/`require` narrowing operators.** DAG-level nodes that convert `Guardable` to `Required`. | Narrowing operators exist as DAG node types. | M | Open |
| 4 | WS4-4 | **Eliminate 7 silent Skipped coercion sites.** Convert to errors. | Each site errors on Skipped instead of coercing. | L | Open |
| 5 | WS4-5 | **Eliminate evaluator silent behaviors.** 12 operations that silently default instead of erroring. | Zero silent defaults in evaluator. | L | Open |

---

## WS-5: Type DAG Execution (The Full Vision)

**Goal**: Type DAG per-layer operations become actual workflow nodes. Coercion = visible graph nodes.
**Design ref**: `compositional-type-coverage.md` § WS-5
**Blocked by**: WS-3
**Success criteria**: Every coercion is a visible node. Every downcast has validation nodes. `TypeShape::Opaque` trends to zero.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS5-1 | **Coercion insertion at lower time.** Lowerer inserts nodes that add/remove type DAG layers. | Coercion edges become visible graph nodes. | XL | Open |
| 2 | WS5-2 | **Downcast validation nodes.** `as Url` inserts per-layer validation nodes. | Every `as` cast has runtime validation per constraint layer. | L | Open |
| 3 | WS5-3 | **Witness-driven test generation.** Each layer's constraints generate boundary test cases. | L4 witnesses generate test cases automatically. | L | Open |
| 4 | WS5-4 | **TypeShape consumed by emitters.** Replace string matching with `TypeShape` dispatch. | `TypeShape::Opaque` trends to zero. | M | Open |

---

## WS-6: Tool/Workflow Completeness

**Goal**: All .dag files compile and have real bodies.
**Design ref**: `compositional-type-coverage.md` § WS-6, § "tools/ and workflows/"
**Blocked by**: WS-1, WS-2
**Success criteria**: All `.dag` files compile. Zero empty stages. Every `func` with resource use declares `uses`.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS6-1 | **Fix `testgen.dag`.** Declare missing `generate` as extern func, or implement. | `testgen.dag` compiles. | S | Open |
| 2 | WS6-2 | **Fix `deps.dag`.** Declare missing externs or implement. | `deps.dag` compiles. | S | Open |
| 3 | WS6-3 | **Add `uses` declarations.** `makegen`, `pragma`, `build` funcs. | Every `func` with resource use declares `uses`. | S | Open |
| 4 | WS6-4 | **Fill `ci.dag` stage bodies.** Wire 12 stages to tool funcs. | Zero empty stages in `ci.dag`. | L | Open |
| 5 | WS6-5 | **Fill remaining workflow stage bodies.** `gist.dag`, `pragma.dag`, others. | Zero empty stages across all pipelines. | L | Open |

---

## WS-7: Extern Linking

**Goal**: Missing extern symbol = hard error. No fallbacks.
**Design ref**: `compositional-type-coverage.md` § WS-7
**No blockers** — independent.
**Success criteria**: All extern symbols resolve through `Backend` or build fails. Zero passthrough fallbacks. Deterministic receipts.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | WS7-1 | **Phase A: Introduce externs.** `extern func`/`extern asset` in parser/typechecker/lowering. | Extern declarations parse, typecheck, and lower correctly. | L | Open |
| 2 | WS7-2 | **Phase B: Linker with hard errors.** `Backend` resolution. Hard-fail on unresolved. | Unresolved extern = compile error with diagnostic. | L | Open |
| 3 | WS7-3 | **Phase C: Migrate handlers/assets.** Convert to extern declarations. Remove `(module, name)` tables. | Runtime ops declared as extern symbols. | L | Open |
| 4 | WS7-4 | **Phase D: Remove fallback surfaces.** Delete `EMBED_REGISTRY`, `is_makegen_module()`, `UnimplementedPassthrough`, etc. | Zero fallback surfaces remain. | M | Open |
| 5 | WS7-5 | **Phase E: Determinism hardening.** `CompileReceipt` hashes, CI gates, diagnostic ordering. | Compile receipts are deterministic. CI gates verify. | M | Open |

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
