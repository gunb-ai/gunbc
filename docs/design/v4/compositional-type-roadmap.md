# Compositional Type Modeling: Roadmap to Full Coverage

**Status**: Draft
**Date**: 2026-03-04
**Companion**: `presence-algebra-enforcement.md` (audit data), `domain-hard-error-no-fallback-plan.md` (extern linking)

## The Vision

Every type in the system is a **compositional DAG of cause-and-effect steps**, built from primitive operations starting at the bottom (`Identity` → `Validate` → `Wrap` → `Brand` → ...). Two types are compatible when they share a common DAG prefix. Coercion is adding steps (upcast/widen) or explicitly removing them (downcast/narrow). There are no ad-hoc compatibility rules — the graph structure IS the type algebra.

This is the original design intent from `dsl-design.md` Axiom 5 and the Understanding pattern from `the-gunbai`. The infrastructure exists. The wiring is incomplete.

## What Exists Today

### Infrastructure: Real and Load-Bearing

| Component | File | Status |
|-----------|------|--------|
| `Dag<TypeOp>` — types as DAGs | `core/ir/src/type_op.rs` | **Done** — 9 op variants, genuine DAG structure |
| Type constructors | `core/ir/src/type_lib.rs` | **Done** — builds `Dag<TypeOp>` for all core types |
| `TypeRegistry` — named lookup + coercion graph | `core/ir/src/type_registry.rs` | **Done** — BFS coercion path discovery |
| Contract tower (L1-L4) | `core/ir/src/contract.rs` | **Done** — cardinality, base type, predicates, witnesses |
| Lattice algebra traits | `core/ir/src/algebra.rs` | **Done** — PartialOrder, Join/MeetSemilattice, BoundedLattice |
| `Cardinality` interval algebra | `core/ir/src/types.rs` | **Done** — join, meet, product, sum, satisfies, property-tested |
| `TypeShape` structural extractor | `core/ir/src/type_shape.rs` | **Done** — extracts backend-facing shape from DAG |
| `SemanticCarrierKind` | `core/ir/src/types.rs` | **Done** — 11 carrier kinds, fail-closed for unknown |
| Container covariance | `type_registry.rs` | **Done** — `List<Url>` auto-widens to `List<String>` |
| Coercion = graph path | `type_registry.rs` | **Done** — `coercion_path()` returns explicit chain |
| Edge validation (3-level) | `builder.rs` | **Done** — type + semantic + cardinality on every edge |

### Modeling: Strong Foundation, Incomplete Coverage

| Layer | Quality | Key strength | Key gap |
|-------|---------|-------------|---------|
| `Dag<TypeOp>` infrastructure | **Solid** | Algebra proven, DAGs constructed for all core types | Not executed at runtime; not used by DSL typechecker |
| `extdeps/` behavioral models | **Strong** (85%) | Real `OperationBehavior`, honest unknowns, typed errors | Not structurally consumed by services layer |
| `std/types.dag` vocabulary | **Good** (70%) | Rich refinements, branded IDs, sum types | ~20 Timestamp-as-String, ~15 stringly-typed enums, duplicates |
| `services/` transport layer | **Weak** (50%) | Transport blocks + response blocks present | Dead imports, `String`/`Json` where typed shapes exist |
| `tools/` workflows | **Mixed** (60%) | makegen excellent | testgen/deps won't compile |
| `workflows/` pipelines | **Skeleton** (20%) | Topology correct | Nearly all stage bodies empty |

## The Three Gaps Between Vision and Reality

### Gap 1: The Type DAG is Static, Not Executed

The type DAG describes what a type IS — but it's never run. `TypeOp::Transform(Coercion)` exists as a node type but no interpreter evaluates it. Coercion "adding a step" means adding an edge to the registry's static graph, not inserting an actual runtime transform node into the workflow DAG.

**What "types as executable DAGs" would mean**: When the compiler detects that `Url` feeds into `String`, instead of just checking compatibility via BFS, it would insert the actual coercion path as workflow nodes — `Validate(Matches(URL_RE))` nodes would be present in the compiled graph. Downcasts would insert validation nodes. This is the cause-and-effect chain: the type DAG IS the validation workflow.

### Gap 2: Two Type Worlds (DSL vs IR)

The DSL typechecker (`daglang-typecheck`) has its own `RecordTypeRegistry` and string-based type resolution. The IR has `TypeRegistry` with `Dag<TypeOp>`. They're not unified. A type defined in `.dag` goes through the DSL typechecker (string matching) and only becomes a `Dag<TypeOp>` when it enters the IR — if it ever does.

**What unification would mean**: DSL type definitions compile to `Dag<TypeOp>` at parse time. The typechecker walks these DAGs instead of doing string comparison. `T` vs `T?` becomes `Dag<TypeOp>` vs `Dag<TypeOp> + Wrap(Optional)` — structurally unambiguous.

### Gap 3: Consumers Don't Use What Producers Define

The extdeps layer defines `AnthropicModel`, `List<LlmMessage>`, `StopReason`. The services layer imports them and uses `String`, `Json`, `String`. There's no mechanism — lint or compile error — that says "you imported a refined type and used a weaker one for the same semantic concept."

**What enforcement would mean**: If a service operation's `input {}` field has the same name as an extdeps type field, and the extdeps uses a refined type while the service uses `String`, the compiler warns (or errors). The type DAG makes this detectable: `String` and `AnthropicModel` have different DAG structures, and the compiler can see that `AnthropicModel` refines `String` via the coercion path.

## Workstreams

### WS-1: Fix the std/ Primitive Vocabulary

**Goal**: Make `std/` a reference-quality foundation that downstream layers build on.

**Work**:

1. **Timestamp consistency**: Replace ~20 `String` timestamp fields with `Timestamp` across `types.dag`, `resources.dag`
2. **Enum extraction**: Convert ~15 stringly-typed enumerations to sum types (`TopologyNodeKind`, `DocSourceKind`, `SeverityLevel`, `DataSource`, `RetryTrigger`)
3. **Brand application**: Apply `ContentHash` brand to the 3 sites that use `NonEmptyStr`
4. **Duration unit type**: Create `Seconds` and `Milliseconds` branded types; use them consistently
5. **Duplicate resolution**: Merge the two `RetryPolicy` definitions; deduplicate `EntryKind`/`SymlinkTarget`
6. **Missing types**: Add `LanguageId`, `GcpRegion`, canonical error wrapper sum type, C/MIPS/Dag language definitions
7. **Stub cleanup**: Delete or implement the 8 stubs that look like features (per PR-gate invariant)
8. **`Filesystem.read`**: Change `path: String` to `path: TextFilePath` (match comment to signature)

**Success criteria**: Zero `String` fields in `std/types.dag` where a refinement type exists for that semantic concept. Zero duplicate type definitions. Zero stubs without `@testgen_skip` or deletion.

### WS-2: Service Layer Type Discipline

**Goal**: Services use the types their extdeps define.

**Work**:

1. **Dead import audit**: For each service file, either use every import or delete it
2. **Input/output type upgrades**: Replace `String`/`Json` with the domain types already imported (see the concrete list in `presence-algebra-enforcement.md § "Concrete type downgrades")
3. **Behavioral property completion**: Add `readonly` to all GET/list operations, `idempotent` to all PUT/DELETE operations
4. **`auth_input` completion**: Add to `issues.dag`, `pull_request.dag`, `anthropic.dag`, `openai.dag`
5. **`owner`/`repo` as service config params**: Formalize service-level path parameters separate from operation inputs

**Success criteria**: Zero dead imports in `services/`. Zero `Json` success responses where typed shapes exist in extdeps. Every GET declares `readonly`. Every service with `auth: BearerToken` has `auth_input`.

### WS-3: Unify DSL Typechecker with IR TypeRegistry

**Goal**: One type world, not two.

**Work**:

1. **DSL type definitions → `Dag<TypeOp>` at parse time**: When the parser sees `type Url = String @non_empty @pattern("...")`, emit a `Dag<TypeOp>` with `Identity("String") → Validate(NonEmpty) → Validate(Matches("..."))` immediately
2. **Typechecker uses `TypeContract` for compatibility**: Replace string-suffix `normalize_type_id` with `TypeContract::can_safely_coerce_to_with()` — the same 3-level check the builder already uses
3. **Optionality is structural**: `T?` parses to `Wrap(Optional)` in the type DAG, not a string suffix. The typechecker sees `Cardinality::ZERO_OR_ONE` vs `Cardinality::ONE` and rejects incompatible assignments
4. **Branch type unification**: `if/else` and `match` arms compute the `join` (LUB) of their type DAGs via the lattice algebra that already exists
5. **Match exhaustiveness**: `Coproduct` variants are known from the type DAG; the typechecker verifies all variants are covered

**Success criteria**: `daglang-typecheck` does not use `normalize_type_id`. All compatibility checks go through `TypeContract`. `T` and `T?` are not interchangeable. Match exhaustiveness is static.

### WS-4: Presence Axis on Ports

**Goal**: Guard-skippable outputs cannot silently feed required inputs.

**Work**:

1. Add `presence: PresenceMode` to `Port` — `Required | Guardable`
2. Guard skip produces `Value::Skipped` only on `Guardable` output ports
3. `DagBuilder::add_edge` rejects `Guardable → Required` without an explicit narrowing node
4. Add `default(value, fallback)` and `require(value)` as DAG-level narrowing operators
5. Eliminate the 7 silent Skipped coercion sites (error instead of coerce)

**Success criteria**: Zero silent `Skipped → concrete_value` coercions. `Value::Skipped` unreachable on any `Required` port. Every fallback is an explicit graph node.

### WS-5: Type DAG Execution (The Full Vision)

**Goal**: Coercion inserts actual validation/transform nodes into the workflow DAG.

**Work**:

1. **Coercion insertion at lower time**: When an edge connects `Url` output → `String` input, the lowerer inserts the coercion path as actual nodes (unwinding `Validate(Matches)` → `Validate(NonEmpty)` → passthrough). The workflow DAG carries the proof that the coercion is safe.
2. **Downcast validation nodes**: When code does `result as Url`, the lowerer inserts `Validate(NonEmpty)` + `Validate(Matches(URL_RE))` nodes that run at execution time and error on invalid values.
3. **Witness-driven test generation**: The L4 witnesses from `TypeContract` become test inputs automatically — boundary values for `@range`, empty/non-empty for `@non_empty`, pattern match/mismatch for `@pattern`.
4. **TypeShape consumed by emitters**: Replace string matching in `daglang-emit` with `TypeShape` dispatch.

**Success criteria**: Every type coercion in the workflow DAG is a visible node (not a silent compatibility check). Every downcast has a validation node. `TypeShape::Opaque` count trends to zero.

### WS-6: Tool/Workflow Completeness

**Goal**: All .dag files compile and have real bodies.

**Work**:

1. Fix `testgen.dag` (declare missing `generate` extern or implement)
2. Fix `deps.dag` (declare missing `parse_deps_toml`, `shell_check`, `shell_exec`)
3. Add `uses` declarations to `makegen`, `pragma`, `build` funcs
4. Fill workflow stage bodies (at minimum `ci.dag` — the primary pipeline)
5. Wire workflow stages to tool funcs

**Success criteria**: `cargo test` passes with all `.dag` files compilable. Zero workflow stages with empty bodies in `ci.dag`. Every `func` that calls a resource pattern declares `uses`.

## Phasing

```
WS-1 (std/ cleanup)          ████░░░░  ~2 sessions — mechanical, no new infra
WS-2 (service discipline)    ████░░░░  ~2 sessions — mechanical, uses WS-1 types
WS-4 (presence axis)         ██████░░  ~3 sessions — new Port field + validation
WS-3 (typechecker unify)     ████████  ~5 sessions — largest, but algebra exists
WS-6 (tool/workflow bodies)  ████░░░░  ~2 sessions — mechanical
WS-5 (type DAG execution)    ██████████ ~8 sessions — the full vision

WS-1 → WS-2 (services need the types WS-1 creates)
WS-3 → WS-4 (presence needs the unified typechecker)
WS-3 → WS-5 (execution needs the unified type DAGs)
WS-1,2 → WS-6 (tools/workflows need the vocabulary)
```

WS-1 and WS-2 are prerequisites for everything else and can start immediately. WS-4 is the highest-ROI infrastructure change (eliminates silent fallbacks). WS-3 is the keystone that enables WS-5 (the full vision). WS-5 is the endgame.

## Success Criteria: "The Entire Codebase Speaks This Language"

The system is complete when these properties hold:

1. **Every `.dag` type compiles to a `Dag<TypeOp>`** — not a string label. The typechecker walks the DAG, not string suffixes.

2. **Every coercion is a visible graph node** — not a silent compatibility check. If `Url` feeds `String`, there's a node in the workflow DAG that proves the coercion is safe. If `String` feeds `Url`, there's a validation node that checks the invariants.

3. **Every absence is typed** — `Required` vs `Guardable` vs `Optional` on every port. No `Value::Skipped` reaches a `Required` input. Every fallback is an explicit `default`/`require` node.

4. **Every service uses its extdeps types** — zero `String` where a refined type is imported and in scope. Zero `Json` escape hatches for structured responses. Zero dead imports.

5. **Every behavioral property is declared** — `readonly` on reads, `idempotent` on idempotent operations, `hermetic` on local-only operations. The compiler derives test classification, retry eligibility, and resource conflict analysis from these — not from heuristics.

6. **Every stub is either implemented or deleted** — per the PR-gate invariant "no stubs that look like features."

7. **The Understanding pattern is fully ported** — extdeps behavioral models (`OperationBehavior` with `failure_modes`, `edge_cases`, `unknowns`, `confidence`) are consumed by the compiler to generate mock specs, error classifiers, and contract tests. Not just documentation.

When all seven hold, the type algebra is the single source of truth for correctness — if the DAG validates, the wiring is right. That's the original promise of "everything is a DAG."
