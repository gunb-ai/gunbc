# Compositional Type Coverage

**Status**: Draft
**Date**: 2026-03-04

## The Principle

**Decisions obligate. Obligations propagate. Propagation is automatic.**

A type is a chain of decisions. Each decision creates obligations that automatically constrain everything downstream. You cannot make a decision without accepting its consequences, and you cannot consume a type without discharging the obligations its decisions created.

- `Wrap(Optional)` = the decision "this might be absent." The obligation: every consumer must handle absence explicitly (`default`, `require`, `match Some/None`). The obligation cannot be ignored — attempting to use the value as-if-present is a compile error.
- `Wrap(List)` = the decision "there may be many." The obligation: every consumer must handle multiplicity (iterate, aggregate, take first). Treating a list as a scalar is a compile error.
- `Validate(NonEmpty)` = the decision "this is never empty." The obligation: to regain this proof after widening to `String`, you must re-validate. The proof does not survive coercion — it must be re-earned.
- `Brand("CommitSha")` = the decision "this means commit SHA, not arbitrary string." The obligation: using it where plain `String` is expected requires explicitly acknowledging the semantic drop.
- `Coproduct(variants)` = the decision "this is one of N alternatives." The obligation: every consumer must handle all N. Missing a variant is a compile error.

The principle is tautological: the description IS the enforcement. You can't say "this is optional" without simultaneously saying "consumers must handle absence." They're the same statement from two directions. There is no state where you've "forgotten" to handle optionality or "accidentally" dropped a constraint, because the obligation is structurally present in the DAG and the system won't let you proceed without discharging it.

### The same principle at every level

This principle — decisions obligate, obligations propagate — applies at three levels in the system. Each is the same pattern: a property declared at one node creates obligations that flow to every downstream consumer.

**Level 1: Type structure** (this doc's primary focus)

`Wrap(Optional)` on a type → every consumer must handle absence. `Coproduct(variants)` → every consumer must handle all variants. The type DAG carries the obligation; the compiler enforces discharge.

**Level 2: Behavioral properties** (the-gunbai Understanding pattern, partially ported)

`readonly` on a service operation → every callable that transitively calls it inherits the property. `idempotent` → same. A `func` that calls ANY non-readonly service becomes non-readonly. The obligation propagates upward through the call graph.

This is already partially implemented: `CallableProperties` in `daglang-derive` does a BFS from each callable entrypoint, collecting `idempotent` and `readonly` from all reachable `ServiceCallMetadata`. The composite property is pessimistic — one non-readonly child makes the parent non-readonly.

**What's missing**: the propagation is collect-only, not enforced. If a `func` is declared `readonly` but calls a non-readonly service, there is no compile error. The obligation exists (readonly = "I promise I don't write state") but no validation checks that the promise is kept. The obligation should propagate like type obligations:

```
error[E5001]: readonly violation
  --> tools/gist.dag:12:3
   |
12 |   func snapshot() readonly {
   |                   ^^^^^^^^ declared readonly
...
15 |     gist.Create(...)
   |     ^^^^^^^^^^^ calls non-readonly operation `gist.Create`
   |
   = note: readonly obligation requires all transitive calls to be readonly
   = help: remove `readonly` or replace with a read-only operation
```

**Level 3: Behavioral contracts** (the-gunbai Understanding pattern, not yet ported)

In the-gunbai, each Understanding declares behavioral properties with explicit propagation:

```
OperationBehavior {
    side_effects: WritesState,
    idempotent: true,
    idempotency_keys: ["gist_id"],
    determinism: NonDeterministic,
    failure_modes: [...],
    unknowns: ["Exact retry-after header behavior under sustained load"],
    confidence: HighConfidence,
}
```

This is richer than boolean `readonly`/`idempotent` — it carries the *reason* (`idempotency_keys`), the *uncertainty* (`unknowns`), and the *confidence level*. Each property creates obligations:

- `idempotent: true` with `idempotency_keys: ["gist_id"]` → the caller must pass the same `gist_id` for retry safety. The key is an obligation on the caller.
- `determinism: NonDeterministic` → test generation cannot assert exact output equality. The property constrains what test obligations are generated.
- `unknowns: [...]` → the system acknowledges it doesn't fully understand this behavior. A confidence-sensitive consumer might reject operations with low confidence.

The extdeps layer already has `OperationBehavior` declarations (see `dsl/std/behavioral.dag`). They're data — not consumed by the compiler for obligation enforcement. The full vision: the compiler reads `OperationBehavior`, propagates properties through the call graph, and generates test obligations, retry policies, and error classifiers from them automatically. Not just documentation — structural obligations.

**The pattern across all three levels**:

```
Level 1 (type):      Wrap(Optional)      → consumer must handle absence
Level 2 (behavior):  readonly            → caller must not write state
Level 3 (contract):  idempotent(keys)    → caller must pass same keys for retry

Same principle:      decision at node     → obligation on consumer
                     obligation missed    → compile error
                     no silent discharge  → explicit opt-out required
```

### The vision

Every type is a compositional DAG of cause-and-effect steps, built from primitive operations (`Identity` → `Validate` → `Wrap` → `Brand` → ...). Two types are compatible when they share a common DAG prefix. Coercion is adding steps (upcast/widen) or explicitly removing them (downcast/narrow). There are no ad-hoc compatibility rules — the graph structure IS the type algebra.

The same structure applies to behavioral properties and contracts: they are DAG nodes whose obligations propagate downstream. The type DAG, the behavioral property graph, and the contract obligations are all instances of the same compositional principle.

This is the original design intent from `dsl-design.md` Axiom 5 and the Understanding pattern from `the-gunbai`. The infrastructure exists at all three levels. The wiring is incomplete. Appendix A shows concrete examples of where the principle is violated today and what compiler diagnostics would catch them.

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

### The node contract: three required dimensions per layer

Every node in the type DAG must explicitly declare three dimensions as a baseline:

- **Base type** — what kind of thing at this level (`Set`, `Inherited`, or a specific type)
- **Cardinality** — how many at this level (`Set([0,∞])`, `Inherited`)
- **Predicates** — what constraints at this level (`Add(NonEmpty)`, `Inherited`)

`Inherited` means "unchanged from the layer below — pass through." `Set` means "this layer sets the value." `Add` means "this layer adds to the set."

This is the contract between layers. Each node must declare all three — you can't add a new node type and silently leave cardinality unspecified. The compiler forces every layer to be explicit about what it contributes:

```
Identity("String") {
    base_type:    Set(String),     // this is a String
    cardinality:  Set([1,1]),      // exactly one
    predicates:   Inherited,       // no constraints yet
}

Validate(NonEmpty) {
    base_type:    Inherited,       // still a String
    cardinality:  Inherited,       // still exactly one
    predicates:   Add(NonEmpty),   // adds non-empty constraint
}

Validate(Pattern("^https?://")) {
    base_type:    Inherited,       // still a String
    cardinality:  Inherited,       // still exactly one
    predicates:   Add(Pattern),    // adds URL pattern constraint
}

Brand("Url") {
    base_type:    Inherited,       // still a String structurally
    cardinality:  Inherited,       // still exactly one
    predicates:   Add(Brand),      // adds semantic brand
}

Wrap(Optional) {
    base_type:    Inherited,       // wraps whatever is below
    cardinality:  Set([0,1]),      // this layer: zero or one
    predicates:   Inherited,       // no new constraints
}

Wrap(List) {
    base_type:    Inherited,       // wraps whatever is below
    cardinality:  Set([0,∞]),      // this layer: zero or more
    predicates:   Inherited,       // no new constraints
}
```

### Compatibility = parallel DAG walk, comparing node contracts

Two types are compatible when you can walk both DAGs layer-by-layer and every node contract widens (or stays the same). At each layer, three things must widen simultaneously:

1. **Base type widens** — `Url` → `String` is ok (dropping refinement). `String` → `Url` is not (adding constraints = narrowing).
2. **Cardinality widens** — `[1,1]` → `[0,1]` is ok (required feeds optional). `[0,1]` → `[1,1]` is not (optional feeds required — value might be absent).
3. **Predicates drop** — `@non_empty @pattern(...)` → `@non_empty` is ok (dropping a constraint). Adding a constraint narrows.

This check happens **at every layer**, not once on the port. Connecting `List<Optional<Url>>` → `List<String>` requires comparing the node contract at each position:

```
Layer 3:  List ↔ List              ← cardinality [0,∞] ↔ [0,∞] ✓
Layer 2:  Optional → ???           ← [0,1] dropped! Was the inner value optional? Is the
                                     target expecting a required value? Must check.
Layer 1:  Url → String             ← predicates drop (NonEmpty, Pattern removed) ✓
Layer 0:  String ↔ String          ← base match ✓
```

If any layer's node contract narrows in a direction the target doesn't expect, it's a compile error — with a diagnostic identifying *which layer* failed.

Because the contracts are explicit on every node (not reverse-engineered by walking ancestors), the per-layer comparison is trivial: just zip the two DAGs and compare contracts at each position.

### The `TypeOp` vocabulary

Each variant is a node type that carries the three-dimension contract:

```
TypeOp = Identity(type)      — sets base type (starting point)
       | Validate(Pred)       — adds a predicate constraint (narrows)
       | Transform(Coercion)  — converts between types (coercion step)
       | Wrap(Optional|List|Set) — sets cardinality at this layer
       | Unwrap               — removes a cardinality layer
       | Product(fields)      — record: named fields, each with its own type DAG
       | Coproduct(variants)  — sum type: tagged alternatives
       | Brand(name)          — adds a semantic tag (same structure, different meaning)
       | Invariant(pred)      — structural invariant
```

Type checking, coercion insertion, and test generation all operate on this same DAG structure. The node contract ensures every variant is explicit about its effect on all three dimensions.

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
| Contract extraction (per-layer) | `core/ir/src/contract.rs` | **Done** — reverse-engineers cardinality, base type, predicates, witnesses from `Dag<TypeOp>` by walking the DAG. To be replaced by explicit node contracts. |
| Lattice algebra traits | `core/ir/src/algebra.rs` | **Done** — PartialOrder, Join/MeetSemilattice, BoundedLattice |
| `Cardinality` interval algebra | `core/ir/src/types.rs` | **Done** — join, meet, product, sum, satisfies, property-tested |
| `TypeShape` structural extractor | `core/ir/src/type_shape.rs` | **Done** — extracts backend-facing shape from DAG |
| `SemanticCarrierKind` | `core/ir/src/types.rs` | **Done** — 11 carrier kinds, fail-closed for unknown |
| Container covariance | `type_registry.rs` | **Done** — `List<Url>` auto-widens to `List<String>` (walks one layer) |
| Coercion = graph path | `type_registry.rs` | **Done** — `coercion_path()` returns explicit chain |
| Edge validation | `builder.rs` | **Partial** — checks type + semantic + cardinality, but as a **flat check on the outermost port**, not a per-layer DAG walk |

**Two gaps**: (1) The edge validator and DSL typechecker only check the outermost layer — the per-layer DAG walk is not yet wired. (2) The node contracts (base type, cardinality, predicates) are reverse-engineered by `TypeContract` after the fact, rather than declared explicitly on each `TypeOp` node. Explicit node contracts would make per-layer comparison trivial and fail-closed for new node types.

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

**Goal**: One type world. Compatibility checking walks the type DAG per-layer, comparing explicit node contracts, instead of comparing strings. Behavioral properties validated against call graph. Behavioral contracts consumed for obligation generation.

1. **Explicit node contracts on `TypeOp`** — each `TypeOp` variant declares its effect on all three dimensions (base type, cardinality, predicates) as `Set`/`Add`/`Inherited`. Replaces `TypeContract` reverse-engineering. New node types must declare all three — fail-closed by construction.
2. DSL type definitions → `Dag<TypeOp>` at parse time (each type becomes a layered DAG with explicit contracts)
3. Typechecker uses per-layer node contract comparison for compatibility (replaces string-based `normalize_type_id`)
4. Optionality is a DAG layer (`T?` → `Wrap(Optional)` with cardinality `Set([0,1])`, not a string suffix)
5. Branch type unification — `if/else` and `match` compute `join` (LUB) of type DAGs per-layer
6. Match exhaustiveness — `Coproduct` variants known from type DAG, checked statically
7. **Behavioral property enforcement (Level 2)** — validate `readonly`/`idempotent` declarations against `CallableProperties` BFS results. `func snapshot() readonly` that calls a non-readonly service = compile error `E5001`. Infrastructure exists (`daglang-derive` BFS); validation pass missing.
8. **Behavioral contract consumption (Level 3)** — compiler reads `OperationBehavior` from extdeps (`idempotency_keys`, `determinism`, `failure_modes`). Generates retry constraints from `idempotency_keys` (`E5003`), test constraints from `determinism`, error classifier hints from `failure_modes`.

**Done when**: `normalize_type_id` deleted. Every `TypeOp` carries an explicit node contract. All checks walk type DAGs per-layer comparing node contracts. `T`/`T?` not interchangeable (different cardinality layer). Exhaustiveness is static. `readonly`/`idempotent` declarations validated against derived properties. `OperationBehavior` consumed for obligation generation.

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

5. **Every behavioral property is declared and enforced** — `readonly` on reads, `idempotent` on idempotent operations, `hermetic` on local-only. Declarations validated against `CallableProperties` BFS — contradiction is a compile error (`E5001`). The compiler derives test classification, retry eligibility, and resource conflict analysis from these.

6. **Every stub is either implemented or deleted** — per the PR-gate invariant.

7. **Every extern symbol resolves or the build fails** — no passthrough fallbacks, no stub asset fallbacks, no module-name dispatch tables. Deterministic compile receipts.

8. **Behavioral contracts generate obligations** — `OperationBehavior` from extdeps consumed by the compiler. `idempotency_keys` checked at retry sites (`E5003`). `determinism` constrains test assertions. `failure_modes` inform error classifiers. Not just documentation — structural obligations.

When all eight hold, the type algebra is the single source of truth for correctness — if the DAG validates, the wiring is right.

---

## Appendix A: Worked Examples — Violations and Expected Diagnostics

Each example shows a real pattern from the codebase where the "decisions obligate" principle is violated, what the violation is, and what compiler diagnostic we expect to see when the system is complete.

### A.1: Skipped value used as if present (obligation: handle absence)

**Current code** (`eval.rs:field_access`):
```rust
Value::Unit | Value::Skipped => Ok(Value::Unit),  // accessing .field on Skipped → Unit
```

**The violation**: `Value::Skipped` means "this port was not wired — the value is structurally absent." Accessing `.field` on it should be impossible, not silently produce `Unit`. The decision to skip (via guard) created an obligation: consumers must handle absence. This code discharges the obligation by pretending the value is `Unit`.

**Expected diagnostic**:
```
error[E2105]: field access on potentially-absent value
  --> tools/makegen.dag:47:22
   |
47 |     entry.ensure_target
   |     ^^^^^ `entry` has type `MakefileTarget?` (cardinality [0,1])
   |
   = help: use `entry?.ensure_target` or `match entry { Some(e) => e.ensure_target, None => ... }`
   = note: obligation from Wrap(Optional) at layer 2 not discharged
```

This applies to all 7 silent coercion sites. The pattern is the same: `Skipped`/`Unit` is treated as a concrete value instead of forcing the consumer to handle the `[0,1]` cardinality layer.

### A.2: `String` where `Timestamp` is the semantic type (obligation: use what you defined)

**Current code** (`types.dag`):
```
type Timestamp = String where pattern("^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}")

type ClaimLease {
  acquired_at: String       // ← Timestamp defined but not used
  expires_at:  String       // ← same
}
```

**The violation**: `Timestamp` is `String + Validate(NonEmpty) + Validate(Pattern(ISO8601))`. Using `String` instead drops two predicate layers. The type DAG for `acquired_at` is shorter than it should be — it's missing the obligations that `Timestamp` creates (valid format, non-empty). Any code that writes to `acquired_at` can put `"hello"` or `""` in it.

**Expected diagnostic**:
```
warning[W1001]: field uses weaker type than available refinement
  --> std/types.dag:642:3
   |
642|   acquired_at: String
   |   ^^^^^^^^^^^ `String` used, but `Timestamp` exists for this semantic concept
   |
   = note: `Timestamp` adds Validate(NonEmpty) + Validate(Pattern) layers
   = help: change to `acquired_at: Timestamp`
```

### A.3: Service imports refined type, uses `String` (obligation: consume what you import)

**Current code** (`anthropic.dag`):
```
import extdeps.llm.anthropic { AnthropicModel, ContentBlock, SystemPrompt, ... }

operation Messages {
  input {
    model: String       // ← AnthropicModel imported and in scope
    messages: Json      // ← LlmMessage imported and in scope
    system: String?     // ← SystemPrompt imported and in scope
  }
  output {
    stop_reason: String from "stop_reason"  // ← StopReason imported and in scope
  }
}
```

**The violation**: `AnthropicModel` is `String + Brand("AnthropicModel")` — it carries a brand layer that `String` doesn't. Using `String` drops the brand obligation: any string is accepted, not just valid model identifiers. `StopReason` is a `Coproduct` (sum type) — using `String` drops the exhaustiveness obligation: callers can't `match` on it without a runtime parse.

**Expected diagnostics**:
```
error[E6001]: imported type not used at service boundary
  --> services/llm/anthropic.dag:8:3
   |
 8 | import extdeps.llm.anthropic { AnthropicModel, ... }
   |                                 ^^^^^^^^^^^^^^ imported but not used in any operation
   |
   = note: `AnthropicModel` refines `String` via Brand("AnthropicModel")

error[E6002]: weaker type used where refined import exists
  --> services/llm/anthropic.dag:18:5
   |
18 |     model: String
   |     ^^^^^ `String` used, but imported `AnthropicModel` (String + Brand) is in scope
   |
   = help: change to `model: AnthropicModel`
   = note: dropping Brand layer removes semantic obligation — callers can pass any String

error[E6003]: opaque Json where structured type exists
  --> services/llm/anthropic.dag:19:5
   |
19 |     messages: Json
   |     ^^^^^^^^ `Json` used, but `List<LlmMessage>` is available
   |
   = note: `Json` has zero layers (opaque). `List<LlmMessage>` has 4 layers
           (Identity(LlmMessage) → Product(role, content) → Wrap(List))
   = help: change to `messages: List<LlmMessage>`
```

### A.4: `first()` on empty list returns `Unit` silently (obligation: handle multiplicity→singleton)

**Current code** (`eval.rs`):
```rust
PipeMethod::First => match receiver {
    Value::List(items) => Ok(items.into_iter().next().unwrap_or(Value::Unit)),
    ...
},
```

**The violation**: `first()` removes a `Wrap(List)` layer (cardinality `[0,∞]`) and should produce an `Optional` (cardinality `[0,1]`). Returning `Value::Unit` is the correct runtime representation of `None`, but the type system doesn't track that the result is now optional. Downstream code accesses `.field` on it without handling absence.

**Current usage** (`makegen.dag`):
```
let entry = matches |> first()
match entry {
  None => ""
  _ => entry.ensure_target    // ← accessing .field on a value that might be Unit
}
```

This works by accident — `None` matches `Value::Unit` (which `first()` returns for empty), and `_` catches the actual value. But the `_` arm accesses `entry.ensure_target` directly without unwrapping a `Some`. If `first()` returned a proper `Optional<MakefileTarget>`, the arm would need `Some(e) => e.ensure_target`.

**Expected diagnostic**:
```
error[E2105]: field access on potentially-absent value
  --> tools/makegen.dag:47:22
   |
45 |   let entry = matches |> first()
   |       ----- type is `MakefileTarget?` (cardinality [0,1] from first())
47 |     _ => entry.ensure_target
   |          ^^^^^ accessing field on value with [0,1] cardinality
   |
   = help: use `Some(e) => e.ensure_target` to unwrap the Optional layer
   = note: `first()` removes Wrap(List) and adds Wrap(Optional) — obligation to handle absence
```

### A.5: `if` without `else` silently produces `Unit` (obligation: both branches must return)

**Current code** (`eval.rs`):
```rust
LoweredExpr::IfElse { cond, then_, else_ } => {
    let condition = eval_expr(cond, env, sibling_fns)?;
    if value_truthy(&condition) {
        eval_expr(then_, env, sibling_fns)
    } else if let Some(else_branch) = else_ {
        eval_expr(else_branch, env, sibling_fns)
    } else {
        Ok(Value::Unit)  // ← no else → Unit
    }
}
```

**The violation**: `if cond { expr }` without `else` has two possible outcomes: the `then` branch value or `Unit`. This is an implicit `Optional` — the result has cardinality `[0,1]`. But the type system doesn't add a `Wrap(Optional)` layer, so downstream code treats the result as `[1,1]` (always present).

**Expected diagnostic**:
```
error[E3001]: if-expression without else has Optional return type
  --> tools/pragma.dag:23:5
   |
23 |   if needs_update { update_file(path) }
   |   ^^^ expression returns `Unit` when condition is false
   |
   = note: result type is `T?` (cardinality [0,1]), not `T` (cardinality [1,1])
   = help: add `else { ... }` to make both branches explicit,
           or use `let result: T? = if ...` to acknowledge optionality
```

### A.6: Match with `_` on known sum type (obligation: handle all variants)

**Current code** (`state_machines.dag`):
```
fn is_legal_forward(from: IssueLifecycleStage, to: IssueLifecycleStage) -> Bool {
  match from {
    Idea         => to == Design
    Design       => to == DesignReview
    DesignReview => to == Accepted
    Accepted     => to == Implementing
    Implementing => to == CodeReview
    CodeReview   => to == Testing
    Testing      => to == Done
    _            => false    // ← Done + TerminalFailed collapsed
  }
}
```

**The violation**: `IssueLifecycleStage` is a `Coproduct` with 9 variants. The `Coproduct` layer creates an exhaustiveness obligation — every variant must be handled. The `_ => false` arm discharges this obligation silently. If a 10th variant `Deploying` is added, this function silently returns `false` for it instead of producing a compile error.

**Expected diagnostic**:
```
error[E3005]: non-exhaustive match on sum type `IssueLifecycleStage`
  --> std/state_machines.dag:72:3
   |
72 |   match from {
   |   ^^^^^ missing variants: `Done`, `TerminalFailed`
   |
   = note: `IssueLifecycleStage` is Coproduct with 9 variants — exhaustiveness obligation
   = help: add explicit arms for `Done` and `TerminalFailed`, or use
           `_ => false @suppress(exhaustiveness, reason: "terminal states")` to opt out explicitly
```

### A.7: String dispatch where sum type should be used (obligation: parse at the boundary)

**Current code** (`design.dag`):
```
content = match provider {
  "openai" => llm.OpenAI.ChatCompletion(...)
  _ => llm.Anthropic.Messages(...)   // ← "cohere", "", anything → Anthropic
}
```

**The violation**: `provider` is `String` — no `Coproduct` layer, so no exhaustiveness obligation. The `_` arm silently catches every string that isn't `"openai"`, including typos, empty strings, and future provider names. The PR-gate checklist says: "new string-based dispatch → enum at intake, exhaustive match internally."

**Expected diagnostic**:
```
error[E3006]: string dispatch should use sum type
  --> tools/design.dag:42:19
   |
42 |   content = match provider {
   |                   ^^^^^^^^ `provider` is `String` — exhaustiveness cannot be checked
   |
   = note: match has 1 literal arm + wildcard catch-all
   = help: define `type LlmProvider = OpenAI | Anthropic` and change `provider: LlmProvider`
   = note: `Coproduct` layer would create exhaustiveness obligation — new providers
           would be compile errors instead of silent Anthropic fallback
```

### A.8: Behavioral property not enforced (obligation: readonly propagates through call graph)

**Current state** (`daglang-derive`):
```rust
// BFS from callable entrypoint — collects properties pessimistically
if !metadata.readonly {
    *readonly = false;    // one non-readonly child → parent non-readonly
}
```

`CallableProperties` correctly derives that a `func` calling a non-readonly service is itself non-readonly. But there is no validation. A `func` declared `readonly` that calls `gist.Create` (a write operation) produces no error.

**Current code** (`git.dag`):
```
service local.Git {
  operation Status {
    readonly                       // ← declares readonly
    transport shell { argv: ["git", "status", "--porcelain"] }
    ...
  }
  operation CommitAll {
                                   // ← NOT readonly (writes state)
    transport shell { argv: ["git", "add", "-A"] }
    ...
  }
}
```

If a `func` declared `readonly` calls both `Git.Status` (readonly) and `Git.CommitAll` (not readonly), the system computes `readonly = false` for the callable but never checks whether that contradicts the declaration.

**The violation**: The `readonly` declaration is a decision that creates an obligation: all transitive calls must be readonly. The `CallableProperties` BFS correctly propagates the property, but never validates the result against the declaration. The obligation exists but is not enforced.

**Expected diagnostic**:
```
error[E5001]: readonly obligation violated
  --> tools/snapshot.dag:5:3
   |
 5 |   func take_snapshot() readonly {
   |                         ^^^^^^^^ declared readonly
...
 8 |     git.CommitAll(message: "snapshot")
   |     ^^^^^^^^^^^^^^ calls non-readonly operation `local.Git.CommitAll`
   |
   = note: readonly propagates pessimistically — one non-readonly call
           makes the entire callable non-readonly
   = help: remove `readonly` declaration, or replace with a readonly operation
```

### A.9: Behavioral contract not consumed (obligation: idempotency keys must be honored)

**Current state** (`extdeps/cloud/gcp/storage.dag`):
```
behavior update_object_cas {
  side_effects: WritesState
  idempotent: true
  idempotency_keys: ["bucket", "object", "generation"]
  determinism: Deterministic
}
```

This says: `update_object_cas` is idempotent IF AND ONLY IF the caller passes the same `bucket`, `object`, and `generation` values on retry. The `idempotency_keys` field is an obligation on the caller — retry is only safe if those keys match.

**The violation**: `idempotency_keys` is data in the extdeps layer. The compiler never reads it. A retry wrapper that re-calls `update_object_cas` without preserving `generation` would silently produce a non-idempotent retry — violating the contract the extdeps layer declared.

**Expected diagnostic**:
```
error[E5003]: idempotency contract not satisfied
  --> tools/deploy.dag:22:5
   |
22 |     retry(attempts: 3) {
   |     ^^^^^ retries `gcs.UpdateObject` which requires idempotency keys
   |
   = note: `update_object_cas` declares idempotent: true with
           idempotency_keys: ["bucket", "object", "generation"]
   = help: ensure retry passes the same values for bucket, object, generation
           across all attempts, or mark retry as `@unsafe_retry`
```
