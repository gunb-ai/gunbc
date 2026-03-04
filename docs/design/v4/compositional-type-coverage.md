# Compositional Type Coverage

**Status**: Draft
**Date**: 2026-03-04

## The Vision

Every type in the system is a **compositional DAG of cause-and-effect steps**, built from primitive operations starting at the bottom (`Identity` → `Validate` → `Wrap` → `Brand` → ...). Two types are compatible when they share a common DAG prefix. Coercion is adding steps (upcast/widen) or explicitly removing them (downcast/narrow). There are no ad-hoc compatibility rules — the graph structure IS the type algebra.

This is the original design intent from `dsl-design.md` Axiom 5 and the Understanding pattern from `the-gunbai`. The infrastructure exists. The wiring is incomplete.

---

## 1. What the Design Specified

### Types are compositional DAGs (dsl-design.md Axiom 5)

Every type is a chain of operations, each adding one constraint or transformation:

```
String → Validate(NonEmpty) → Validate(Pattern("^https?://")) → Brand("Url")
```

This is the type DAG for `Url`. It says: start with `String`, require it to be non-empty, require it to match a URL pattern, then brand it as `Url`. Each step narrows the set of valid values.

Composed types add more layers:

```
List<Optional<Url>>  =

Layer 0:  String                              ← base type ("what kind of thing")
Layer 1:  Validate(NonEmpty), Validate(Pattern) ← predicates ("what constraints")
Layer 2:  Wrap(Optional)                       ← cardinality: [0,1] ("can it be absent?")
Layer 3:  Wrap(List)                           ← cardinality: [0,∞] ("how many?")
```

Each layer carries three concerns:
- **What kind of thing** — the base type at that level (String, Url, List, etc.)
- **How many** — the cardinality at that level (exactly one, zero-or-one, zero-or-more, etc.)
- **What constraints** — the predicates at that level (@non_empty, @pattern, @range, etc.)

### Compatibility = parallel DAG walk

Two types are compatible when you can walk both DAGs layer-by-layer and every layer widens (or stays the same). At each layer, three things must widen simultaneously:

1. **Base type widens** — `Url` → `String` is ok (dropping refinement). `String` → `Url` is not (adding constraints = narrowing).
2. **Cardinality widens** — `[1,1]` → `[0,1]` is ok (required feeds optional). `[0,1]` → `[1,1]` is not (optional feeds required — value might be absent).
3. **Predicates drop** — `@non_empty @pattern(...)` → `@non_empty` is ok (dropping a constraint). Adding a constraint narrows.

This check happens **at every layer of the composition**, not once on the port. Connecting `List<Optional<Url>>` → `List<String>` requires checking:

```
Layer 3:  List ↔ List              ← cardinality [0,∞] ↔ [0,∞] ✓
Layer 2:  Optional → ???           ← [0,1] dropped! Was the inner value optional? Is the
                                     target expecting a required value? Must check.
Layer 1:  Url → String             ← predicates drop (NonEmpty, Pattern removed) ✓
Layer 0:  String ↔ String          ← base match ✓
```

If any layer narrows in a direction the target doesn't expect, it's a compile error — with a diagnostic identifying *which layer* failed.

### The `TypeOp` vocabulary

```
TypeOp = Identity       — base type (starting point)
       | Validate(Pred)  — add a predicate constraint (narrows)
       | Transform(Coercion) — convert between types (coercion step)
       | Wrap(Optional|List|Set) — add a cardinality layer
       | Unwrap          — remove a cardinality layer
       | Product(fields) — record: named fields, each with its own type DAG
       | Coproduct(variants) — sum type: tagged alternatives
       | Brand(name)     — semantic tag (same structure, different meaning)
       | Invariant(pred) — structural invariant
```

Type checking, coercion insertion, and test generation all operate on this same DAG structure.

### DAG invariants (SPEC.md §2)

| # | Invariant | What it means |
|---|-----------|---------------|
| I2.1 | Acyclicity | No node depends on itself |
| I2.2 | Type agreement | Every edge has matching types at both ends (per-layer check) |
| I2.3 | Port saturation | Every required input has exactly one incoming edge |
| I2.4 | SubDag interface agreement | Parent ports biject onto inner DAG unconnected ports |
| I2.5 | Cardinality honesty | A `One` output cannot feed a `One` input through a conditional path without an explicit merge |
| I2.6 | Explicit opt-out | Silence is a validation error — patterns must be instantiated or declared NotApplicable |

### Extern linking contract

Runtime handlers and embedded assets are linkable extern symbols. Missing symbol resolution is a hard compile/link error. No fallback execution paths.

```rust
pub enum OpRef {
    Intrinsic(IntrinsicOp), // primitives/pattern/transport phases
    Call(SymbolId),         // call DSL-defined symbol
    Extern(SymbolId),       // call extern func (must link)
}
```

Backends resolve symbols via `Backend` trait. Unresolved = hard error.

---

## 2. What Exists Today

### Infrastructure: Real and Load-Bearing

The DAG representation and algebra are built. The per-layer checking is not yet wired through.

| Component | File | Status |
|-----------|------|--------|
| `Dag<TypeOp>` — types as DAGs | `core/ir/src/type_op.rs` | **Done** — 9 op variants, genuine DAG structure |
| Type constructors | `core/ir/src/type_lib.rs` | **Done** — builds `Dag<TypeOp>` for all core types |
| `TypeRegistry` — named lookup + coercion graph | `core/ir/src/type_registry.rs` | **Done** — BFS coercion path discovery |
| Contract extraction (per-layer) | `core/ir/src/contract.rs` | **Done** — extracts cardinality, base type, predicates, witnesses from any `Dag<TypeOp>` |
| Lattice algebra traits | `core/ir/src/algebra.rs` | **Done** — PartialOrder, Join/MeetSemilattice, BoundedLattice |
| `Cardinality` interval algebra | `core/ir/src/types.rs` | **Done** — join, meet, product, sum, satisfies, property-tested |
| `TypeShape` structural extractor | `core/ir/src/type_shape.rs` | **Done** — extracts backend-facing shape from DAG |
| `SemanticCarrierKind` | `core/ir/src/types.rs` | **Done** — 11 carrier kinds, fail-closed for unknown |
| Container covariance | `type_registry.rs` | **Done** — `List<Url>` auto-widens to `List<String>` (walks one layer) |
| Coercion = graph path | `type_registry.rs` | **Done** — `coercion_path()` returns explicit chain |
| Edge validation | `builder.rs` | **Partial** — checks type + semantic + cardinality, but as a **flat check on the outermost port**, not a per-layer DAG walk |

**The gap**: the infrastructure can represent per-layer types and can extract per-layer contracts — but the edge validator and DSL typechecker only check the outermost layer. The per-layer DAG walk that the design calls for is not yet wired.

### Modeling Quality by Layer

```
                    MODELING QUALITY
                    ↑
        extdeps     ████████████████░░  85%  ← strongest layer
        std/        ██████████████░░░░  70%  ← good vocabulary, systematic holes
        tools/      ████████████░░░░░░  60%  ← range: makegen=95%, testgen=broken
        services/   ██████████░░░░░░░░  50%  ← imports types, doesn't use them
        workflows/  ████░░░░░░░░░░░░░░  20%  ← topology only, no bodies
```

The paradox: **the richest type models (extdeps) are consumed by the weakest consumers (services)**. The types exist. The services import them. Then the services use `String` and `Json` anyway.

---

## 3. The Gaps

### Gap 1: Presence — the missing axis (I2.5)

The design says: **"A `One` output cannot feed a `One` input through a conditional path without an explicit merge."** Not enforced.

- Guard skip produces `Value::Skipped` on ALL output ports (regardless of cardinality)
- `Value::Skipped` is a runtime sentinel with no compile-time representation
- 7 sites silently coerce `Skipped` into a concrete value

**What's missing**: `PresenceMode = Required | Guardable` on ports. Builder rejects `Guardable → Required` without an explicit narrowing node.

**Silent coercion sites** (each must become an error):

| File | Function | Coercion |
|------|----------|----------|
| `daglang-lower/src/eval.rs` | `value_to_string` | `Skipped → ""` |
| `daglang-lower/src/eval.rs` | `value_truthy` | `Skipped → false` |
| `daglang-lower/src/eval.rs` | `values_equal` | `Skipped == Unit → true` |
| `daglang-lower/src/eval.rs` | `field_access` | `Skipped.field → Unit` |
| `daglang-lower/src/eval.rs` | `sort_key` | `Skipped → "skipped"` |
| `core/exec/src/execute/mod.rs` | `collect_fan_in` | `Skipped → dropped from list` |
| `core/exec/src/pattern_op.rs` | `list_values` | `Skipped → vec![]` |

### Gap 2: Optionality — string suffix, not a DAG layer

`Port::is_optional()` is `type_id.0.ends_with('?')`. The typechecker's `normalize_type_id` doesn't distinguish `T` from `T?`. This is a flat string check — it doesn't see `Optional` as a `Wrap` layer in the type DAG with its own cardinality `[0,1]`.

**What's missing**: `T?` parses to `Wrap(Optional)` in the type DAG. The typechecker walks the DAG and sees a `[0,1]` cardinality layer. Connecting `Optional<Url>` → `Url` fails because the inner layer narrows from `[0,1]` to `[1,1]` — the value might be absent.

### Gap 3: Branch/match type unification

- `if/else` branches can return different types — no unification
- `match` arms can return different types — no unification
- `match` exhaustiveness is runtime-only — no static check against known sum type variants

**What's missing**: Branch arms compute `join` (LUB) via the lattice algebra. `Coproduct` variants drive static exhaustiveness.

### Gap 4: Two type worlds (DSL vs IR)

The DSL typechecker (`daglang-typecheck`) has its own `RecordTypeRegistry` and string-based type resolution. The IR has `TypeRegistry` with `Dag<TypeOp>`. They're not unified. A type defined in `.dag` goes through the DSL typechecker (string matching) and only becomes a `Dag<TypeOp>` when it enters the IR — if it ever does.

### Gap 5: The type DAG is static, not executed

The type DAG describes what a type IS — but it's never run. When the compiler detects that `Url` feeds `String`, it checks compatibility via BFS on a static graph — it doesn't insert the actual coercion steps (dropping `Validate(Pattern)`, dropping `Validate(NonEmpty)`) as visible workflow nodes. The per-layer DAG walk happens at check time but produces no runtime artifact. In the full vision, coercion = inserting the type DAG's operations as actual workflow nodes that execute.

### Gap 6: Consumers don't use what producers define

The extdeps layer defines `AnthropicModel`, `List<LlmMessage>`, `StopReason`. The services layer imports them and uses `String`, `Json`, `String`. No lint or compile error enforces refined type usage.

### Gap 7: Extern symbols not linked

Runtime handlers use string-coupled `(module, name)` dispatch tables. Embedded assets use stub fallbacks. There is no link step that validates all extern symbols resolve.

### Gap 8: Evaluator silent behaviors

12 operations silently produce defaults instead of erroring:

| Operation | Silent behavior | Should be |
|-----------|----------------|-----------|
| `div/mod by zero` | Returns `0` | Error |
| `map/filter/flat_map` missing lambda | Returns list unchanged | Error |
| `sum` on non-Int elements | Filters silently | Error |
| `join` non-Str separator | Defaults to `","` | Error |
| `contains` no needle | Returns `false` | Error |
| `sort_by` key eval error | Uses `""` | Propagate error |
| `for` on scalar | Wraps in `[scalar]` | Error |
| `first/last` on empty list | Returns `Unit` | Return `Optional` |
| Uppercase unbound ident | `Value::Str(name)` heuristic | Validate against known variants |
| Field access on JSON, missing field | `Json(Null)` | Error |
| `if` without `else` | `Unit` | Require `else` or return `Optional` |
| Match non-exhaustive | Runtime error | Static exhaustiveness check |

---

## 4. DSL File Audit

### std/ Primitives

**Strong**: Refinement types used extensively (`CommitSha`, `HttpStatus`, `Email`, `Port`), sum types with payloads, SDLC domain model comprehensive, state machine module clean, fermi/fidelity vocabulary correct.

**Systematic holes**:

1. **~20 Timestamp-as-String fields** — `Timestamp` defined but not used for `created_at`, `updated_at`, `acquired_at`, `expires_at`, `produced_at`, `consumed_at`, `timestamp` (AuditEntry), Clock.now
2. **~15 stringly-typed enums** — `TopologyNode.kind`, `DocSource.kind`, `DesignFinding.severity`, `DesignOutput.source`, `IssueBinding.provider`, `PipelineArtifact.kind`, `RetryPolicy.retry_on`
3. **`ContentHash` brand not applied** — 3 fields use `NonEmptyStr` instead
4. **Duration unit ambiguity** — milliseconds vs seconds mixed, no unit-branded types
5. **Duplicates** — `RetryPolicy` (2 definitions, different structures), `EntryKind`/`SymlinkTarget` (types.dag + filesystem.dag)
6. **Missing types** — `SeverityLevel`, `DataSource`, `RetryTrigger`, `LanguageId`, `GcpRegion`, canonical error wrapper, C/MIPS/Dag language defs
7. **8 stubs that look like features** — empty `retry<Op>`, always-false `check_iam_binding`, identity `add_iam_binding`, empty `AuthContext.acquire`, empty `Network`, `Filesystem.read` wrong type, undeclared `fs.stat`
8. **`Filesystem.read`** — takes `String`, comment says `TextFilePath`

### services/ — Type Discipline Gap

Services import extdeps domain types but use `String`/`Json` instead:

| Service | Field | Current | Should be |
|---------|-------|---------|-----------|
| `anthropic.dag` | messages | `Json` | `List<LlmMessage>` |
| `anthropic.dag` | model | `String` | `AnthropicModel` |
| `anthropic.dag` | stop_reason | `String` | `StopReason` |
| `openai.dag` | messages | `Json` | `List<LlmMessage>` |
| `openai.dag` | model | `String` | `OpenAiModel` |
| `pull_request.dag` | files | `Json` | `List<PrFile>` |
| `pull_request.dag` | state | `String` | `PrState` |
| `pull_request.dag` | head_sha | `String` | `CommitSha` |
| `gcp/iam.dag` | bindings | `Json` | `List<GcpBinding>` |

Dead imports in every service file (5-8 unused imports each in git.dag, cargo.dag, pull_request.dag, anthropic.dag, openai.dag).

Missing behavioral properties: `readonly` on GET/list operations, `idempotent` on PUT/DELETE, `auth_input` on 4 services.

### extdeps/ — The Strong Layer (85%)

Genuinely well-modeled: `OperationBehavior` schema, honest epistemic `unknowns`, correct idempotency analysis, sum types where domain requires, Workload Identity Federation, provider-specific edge cases (Anthropic 529).

### tools/ and workflows/ — The Execution Gap

- **Production quality**: makegen.dag (35 DSL fns), codegen.dag, gist.dag
- **Broken**: testgen.dag (undefined `generate`), deps.dag (undeclared `parse_deps_toml`, `shell_check`, `shell_exec`)
- **Missing `uses`**: makegen, pragma, build funcs
- **Empty workflows**: ci.dag (12 stages, all empty), gist.dag (10/12 empty), pragma.dag (7/9 empty)

---

## 5. Workstreams

### Dependency Graph

```
WS-1 (std/ cleanup)  ──→  WS-2 (service discipline)  ──→  WS-6 (tool/workflow)
                                                              ↑
WS-3 (typechecker)   ──→  WS-4 (presence axis)        WS-7 (extern linking)
                      ──→  WS-5 (type DAG execution)
```

WS-1 and WS-3 can start immediately. WS-7 is independent.

### WS-1: Fix the std/ Primitive Vocabulary

**Goal**: Make `std/` a reference-quality foundation.

1. Replace ~20 `String` timestamp fields with `Timestamp`
2. Convert ~15 stringly-typed enumerations to sum types
3. Apply `ContentHash` brand to 3 sites
4. Create `Seconds` and `Milliseconds` branded types
5. Merge duplicate `RetryPolicy`; deduplicate `EntryKind`/`SymlinkTarget`
6. Add missing types (`SeverityLevel`, `DataSource`, `RetryTrigger`, etc.)
7. Delete or implement 8 stubs
8. Fix `Filesystem.read` type

**Done when**: Zero `String` fields where a refinement type exists. Zero duplicates. Zero stubs that look like features.

### WS-2: Service Layer Type Discipline

**Goal**: Services use the types their extdeps define.

1. Dead import audit — use or delete every import
2. Input/output type upgrades — replace `String`/`Json` with domain types
3. Behavioral property completion — `readonly`, `idempotent`
4. `auth_input` completion — 4 services
5. `owner`/`repo` as service config params

**Done when**: Zero dead imports. Zero `Json` escape hatches. Every GET declares `readonly`. Every BearerToken service has `auth_input`.

### WS-3: Unify DSL Typechecker with IR TypeRegistry

**Goal**: One type world. Compatibility checking walks the type DAG per-layer instead of comparing strings.

1. DSL type definitions → `Dag<TypeOp>` at parse time (each type becomes a layered DAG)
2. Typechecker uses per-layer `TypeContract` for compatibility (replaces string-based `normalize_type_id`)
3. Optionality is a DAG layer (`T?` → `Wrap(Optional)` with cardinality `[0,1]`, not a string suffix)
4. Branch type unification — `if/else` and `match` compute `join` (LUB) of type DAGs per-layer
5. Match exhaustiveness — `Coproduct` variants known from type DAG, checked statically

**Done when**: `normalize_type_id` deleted. All checks walk type DAGs per-layer. `T`/`T?` not interchangeable (different cardinality layer). Exhaustiveness is static.

### WS-4: Presence Axis on Ports

**Goal**: I2.5 (cardinality honesty) implemented.

1. Add `presence: PresenceMode` to `Port` — `Required | Guardable`
2. Guard skip produces `Value::Skipped` only on `Guardable` output ports
3. `DagBuilder::add_edge` rejects `Guardable → Required` without narrowing
4. Add `default(value, fallback)` and `require(value)` narrowing operators
5. Eliminate 7 silent Skipped coercion sites + 12 evaluator silent behaviors

**Done when**: Zero silent `Skipped → concrete_value` coercions. `Value::Skipped` unreachable on `Required` ports. Every fallback explicit.

### WS-5: Type DAG Execution (The Full Vision)

**Goal**: The type DAG's per-layer operations become actual workflow nodes. Coercion = inserting/removing layers as visible graph nodes.

1. Coercion insertion at lower time — when `Url` feeds `String`, the lowerer inserts nodes that unwind the type DAG layers (drop `Validate(Pattern)`, drop `Validate(NonEmpty)`)
2. Downcast validation nodes — when `String` feeds `Url` (via `as Url`), the lowerer inserts the type DAG layers as validation nodes (`Validate(NonEmpty)` + `Validate(Pattern)`) that execute and error on invalid values
3. Witness-driven test generation — each layer's constraints generate boundary test cases automatically (empty/non-empty for `@non_empty`, boundary values for `@range`, match/mismatch for `@pattern`)
4. TypeShape consumed by emitters — replace string matching with `TypeShape` dispatch

**Done when**: Every coercion is a visible node (adding/removing DAG layers). Every downcast has validation nodes (one per constraint layer). `TypeShape::Opaque` trends to zero.

### WS-6: Tool/Workflow Completeness

**Goal**: All .dag files compile and have real bodies.

1. Fix `testgen.dag` (declare missing `generate` or implement)
2. Fix `deps.dag` (declare missing externs or implement)
3. Add `uses` declarations to `makegen`, `pragma`, `build`
4. Fill `ci.dag` stage bodies — wire to tool funcs
5. Fill remaining workflow stage bodies

**Done when**: All `.dag` files compile. Zero empty stages. Every `func` with resource use declares `uses`.

### WS-7: Extern Linking

**Goal**: Missing extern symbol = hard error. No fallbacks.

1. **Phase A**: Add `extern func`/`extern asset` to parser/typechecker/lowering. Lower to `OpRef::Extern(SymbolId)`.
2. **Phase B**: Add linker with `Backend` resolution. Hard-fail on unresolved symbols.
3. **Phase C**: Migrate runtime handlers/assets to extern declarations. Remove `(module, name)` tables.
4. **Phase D**: Delete fallback surfaces — `EMBED_REGISTRY`, `embed_layer1_handler_data`, `is_makegen_module()`, `MAKEGEN_STUB_CONTENT`, `allow_unimplemented_passthrough`, `UnimplementedPassthrough`, module/name fallback tables.
5. **Phase E**: Determinism hardening — `CompileReceipt` hashes, CI determinism gates, diagnostic ordering.

**Done when**: All extern funcs/assets resolve through `Backend` or build fails. Zero passthrough fallbacks. Deterministic receipts.

---

## 6. Success Criteria

The system is complete when these properties hold:

1. **Every `.dag` type compiles to a `Dag<TypeOp>`** — not a string label. The typechecker walks the type DAG layer-by-layer. Compatibility is checked at every layer of the composition, not once on the outermost port.

2. **Every coercion is a visible graph node** — not a silent compatibility check. Coercion = adding or removing type DAG layers as actual workflow nodes. If `Url` feeds `String`, there are nodes dropping the constraint layers. If `String` feeds `Url`, there are validation nodes checking each constraint layer.

3. **Every absence is typed** — `Required` vs `Guardable` on every port. No `Value::Skipped` reaches a `Required` input. Every fallback is an explicit `default`/`require` node.

4. **Every service uses its extdeps types** — zero `String` where a refined type is in scope. Zero `Json` escape hatches. Zero dead imports.

5. **Every behavioral property is declared** — `readonly` on reads, `idempotent` on idempotent operations, `hermetic` on local-only. The compiler derives test classification, retry eligibility, and resource conflict analysis from these.

6. **Every stub is either implemented or deleted** — per the PR-gate invariant.

7. **Every extern symbol resolves or the build fails** — no passthrough fallbacks, no stub asset fallbacks, no module-name dispatch tables. Deterministic compile receipts.

8. **The Understanding pattern is fully ported** — extdeps behavioral models are consumed by the compiler to generate mock specs, error classifiers, and contract tests. Not just documentation.

When all eight hold, the type algebra is the single source of truth for correctness — if the DAG validates, the wiring is right.
