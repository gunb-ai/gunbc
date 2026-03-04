# Port Type Algebra: Design Intent vs Current Reality

**Status**: Draft
**Date**: 2026-03-04

## The Question

The DAG modeling system was designed so that **every concern is expressed as compositional structure** — each layer declares its constraints, and the compiler enforces them. The port type algebra (value type × cardinality × predicates) was supposed to be the vehicle for this. We're finding silent errors and fallbacks. This doc asks: **where did the implementation fall short of the design?**

---

## What the Design Specified

### The port type is a triple (SPEC.md §1.3, dsl-design.md §4.1)

```
PortType = ValueType × Cardinality × Predicates
```

- **ValueType**: nominal type in a lattice (`Unit ⊥ … Int, Bool, String, Bytes, Secret … Json ⊤`)
- **Cardinality**: closed interval `[min, max]` on ℕ∪{∞} with lattice (join/meet) + semiring (product/sum)
- **Predicates**: Heyting algebra of refinement constraints (`@non_empty`, `@range`, `@pattern`, `@brand`)

### Three-level coercion theorem (dsl-design.md §4.1)

An edge `A → B` is safe iff all three levels widen simultaneously:

```
L1: A.cardinality ⊆ B.cardinality    (interval containment)
L2: A.base_type ≤ B.base_type        (base type lattice upcast)
L3: A.predicates ⊇ B.predicates      (predicate entailment — more constraints = smaller set)
```

If any level narrows, it's a compile error with a diagnostic identifying which level failed.

### Five DAG invariants (SPEC.md §2)

| # | Invariant | What it means |
|---|-----------|---------------|
| I2.1 | Acyclicity | No node depends on itself |
| I2.2 | Type agreement | Every edge has matching types at both ends |
| I2.3 | Port saturation | Every required input has exactly one incoming edge |
| I2.4 | SubDag interface agreement | Parent ports biject onto inner DAG unconnected ports |
| I2.5 | Cardinality honesty | A `One` output cannot feed a `One` input through a conditional path without an explicit merge |
| I2.6 | Explicit opt-out | Silence is a validation error — patterns must be instantiated or declared NotApplicable |

### Compositional layers (handbook §"Compositional Modeling Philosophy")

Each DSL structural block adds constraints that compose additively:

```
config { endpoint: ..., auth: ... }     → provider-level requirements
transport rest { method: ..., path: ... } → transport class selection
readonly / idempotent / hermetic         → behavioral properties
response { STATUS => TYPE }              → error contracts (future)
```

The compiler generates transport code, mock specs, and test obligations reflecting ALL layers. The workflow author names only the top layer.

### Types are DAGs (dsl-design.md Axiom 5)

```
⟦T⟧ = {v | execute(T.dag, v) succeeds}

TypeOp = Identity | Validate(Predicate) | Transform(Coercion) | Wrap | Unwrap
       | Product(fields) | Coproduct(variants) | Brand(name) | Invariant(pred)
```

Type checking, coercion insertion, and test generation all operate on the same DAG structure.

---

## What's Actually Implemented

### Port struct (core/ir/src/dag.rs)

```rust
pub struct Port {
    pub name: PortName,
    pub type_id: TypeId,           // string-based nominal type
    pub cardinality: Cardinality,  // [min, max] interval — FULLY IMPLEMENTED
    pub(crate) guard: Option<Guard>,  // branch routing only, not presence
    pub resource_access: Option<AccessMode>,
    pub log_detail: Option<LogDetailLevel>,
}
```

### What `DagBuilder::add_edge` checks (builder.rs)

| Check | Status | Notes |
|-------|--------|-------|
| Cycle prevention (generational) | **Done** | By construction, not post-hoc |
| Port existence | **Done** | |
| Type expression syntax | **Done** | Parseable `Optional<T>`, `List<T>`, etc. |
| L2 structural + coercion compatibility | **Done** | `is_compatible` → `can_safely_coerce_to_with` → base type lattice |
| L3 predicate entailment | **Done** | `Predicate::entails` (conservative) |
| Semantic carrier compatibility | **Done** | `SemanticCarrierKind` — fail-closed for unknown types |
| L1 cardinality satisfaction | **Done** | `from.satisfies(to)` — interval containment |
| Fan-in on scalar | **Done** | |
| Fan-in cardinality overflow | **Done** | Minkowski sum of incoming edges |

### What `verify_dag` checks (validate.rs)

| Check | Status | Notes |
|-------|--------|-------|
| SubDag interface bidirectional match | **Done** | Recursive, with type+semantic checks |
| Resource wiring completeness | **Done** | Recursive through nested SubDags |
| Fingerprint uniqueness (C22) | **Done** | Deductive redundancy elimination |
| Required input wiring | **Done** | `cardinality.min > 0` ports (skips `res:`, `tool:`, `__` prefixed) |

### What the typechecker checks (daglang-typecheck)

| Check | Status | Notes |
|-------|--------|-------|
| Type name resolution | **Done** | 34 error variants |
| Generic arity | **Done** | |
| Refinement constraint sanity | **Done** (partial) | Literal ranges only |
| Call arity + named arg validity | **Done** | |
| Interface conformance | **Done** | Field-by-field signature matching |
| Pipeline structural checks | **Done** | |
| Pipeline `when` condition is Bool | **Done** | |
| `fn` purity (no effectful nodes) | **Done** | |

### Cardinality algebra (core/ir/src/types.rs)

**Fully implemented.** The interval model with `join` (LUB), `meet` (GLB), `product`, `sum`, `satisfies` is all there and property-tested. Named constants (`ONE`, `ZERO_OR_ONE`, `ZERO_OR_MORE`, `ONE_OR_MORE`) plus arbitrary intervals. `infer_cardinality` auto-derives from wrapper types (`Optional<T>` → `ZERO_OR_ONE`).

### Semantic carrier algebra (core/ir/src/types.rs)

**Fully implemented.** 11 carrier kinds with fail-closed `UnknownSemantic`. Checked on every edge. Transparent through generics (`Optional<Credential>` → Credential).

---

## The Gap: What the Design Specified but Isn't Implemented

### Gap 1: Presence — the missing axis

The design specifies five invariants. I2.5 (cardinality honesty) says: **"A `One` output cannot feed a `One` input through a conditional path without an explicit merge."**

This is not enforced. There is no mechanism to track whether a port's value may be absent due to control flow. Instead:

- Guard skip produces `Value::Skipped` on ALL output ports (regardless of cardinality)
- `Value::Skipped` is a runtime sentinel with no compile-time representation
- Downstream code cannot distinguish "legitimately empty" from "skipped by guard" from "unwired"
- 7 sites silently coerce `Skipped` into a concrete value (`""`, `vec![]`, `false`, dropped)

**What's missing**: a `PresenceMode` axis on ports (`Required | Guardable`) that the builder validates on edges, so a `Guardable` output cannot feed a `Required` input without an explicit `default`/`require` narrowing node.

**This is the core of I2.5 that we didn't build.**

### Gap 2: Optionality — string suffix, not structural

`Port::is_optional()` is `type_id.0.ends_with('?')`. The typechecker's `normalize_type_id` doesn't distinguish `T` from `T?`. The three-level coercion theorem (L1: cardinality containment) should catch this — `ONE` (required) vs `ZERO_OR_ONE` (optional) — but the typechecker doesn't invoke it for data flow. Only the builder's `add_edge` does, and only if the lowerer faithfully translates DSL `T?` into `ZERO_OR_ONE` cardinality on the port.

**What's missing**: the typechecker should enforce that `T?` expressions produce `ZERO_OR_ONE` cardinality ports, and that `T` expressions require `ONE`. The builder already validates cardinality on edges — the gap is that the lowerer doesn't always set cardinality correctly from the DSL type.

### Gap 3: Branch/match type unification

The design says types are DAGs and the coproduct/product structure should compose. But:

- `if/else` branches can return different types — no unification
- `match` arms can return different types — no unification
- `match` exhaustiveness is runtime-only — no static check against known sum type variants

**What's missing**: the typechecker should unify branch types (compute the join/LUB in the type lattice) and check exhaustiveness against `Coproduct` variant sets.

### Gap 4: Annotation categories 2 and 3

The annotation census (`annotation-to-dag-modeling.md`) classifies 43 annotations into three categories:

| Category | Count | Status |
|----------|-------|--------|
| 1: Working infrastructure | ~26 | **Implemented** — desugars to structure |
| 2: Declared intent, no enforcement | ~4 | **Gap** — `@contract`, `@error_map`, `@retry`, `@requires` parsed but inert |
| 3: Metadata noise | ~13 | **Gap** — should be deleted or migrated to `TypeOp::Meta` |

Category 2 is the compositionality gap. These annotations describe real layer constraints (`@error_map` → error classifier nodes, `@retry` → retry wrapper nodes, `@requires` → resource edges) that should compose into generated code per the design philosophy. Instead they're documentation.

### Gap 5: I2.6 (explicit opt-out) not enforced

The spec says: "patterns must be instantiated or declared `NotApplicable { reason }`. Silence is a validation error."

This is not implemented. A service operation that doesn't declare `@error_map` simply has no error classification — no warning, no `NotApplicable`. The catch-all `_ => {}` in annotation parsing silently drops unknown annotations.

### Gap 6: SubDag cardinality not checked

`validate_single_subdag` checks type identity and semantic carrier across SubDag boundaries, but does NOT check cardinality match. A parent declares `ONE` on an output but the inner boundary is `ZERO_OR_MORE` — not caught.

### Gap 7: Control/TriggerGate edges skip all checks

`DagBuilder::add_edge` only adds `DataFlow` edges with full validation. Control and TriggerGate edges are added via `Dag::add_edge` directly — zero type/cardinality/semantic checks.

---

## How the Gaps Map to the Original Design

| Design principle | Implementation status | Gap # |
|-----------------|----------------------|-------|
| Three-level coercion on every edge | **L1+L2+L3 done** in builder | — |
| Cardinality lattice + semiring | **Fully done** | — |
| Semantic carrier algebra | **Fully done** | — |
| I2.1 Acyclicity | **Done** (generational) | — |
| I2.2 Type agreement | **Done** (builder edge check) | — |
| I2.3 Port saturation | **Partial** (required inputs checked, but only in `verify_dag`, and `res:`/`tool:`/`__` excluded) | 1 |
| I2.4 SubDag interface | **Done** (recursive, type+semantic) | 6 (cardinality gap) |
| I2.5 Cardinality honesty (conditional paths) | **Not done** — no presence tracking | **1** |
| I2.6 Explicit opt-out | **Not done** — silence is not an error | **5** |
| Types are DAGs | **Infrastructure done** (`TypeOp`, `TypeContract`, `Predicate`) — not fully wired through typechecker | 2, 3 |
| Compositional layer constraints | **Transport + auth + config done**. Error/retry/requires/contract not done. | **4** |
| Every annotation desugars to structure | ~60% done (Category 1). ~40% inert or noise. | **4, 5** |

---

## The One-Sentence Summary

The **value type**, **cardinality**, and **semantic carrier** axes of the port type algebra are fully implemented and enforced at compile time. The missing piece is the **presence axis** (I2.5: can this value be absent due to control flow?) — which is currently a runtime sentinel (`Value::Skipped`) with 7 silent coercion sites and no compile-time tracking. The secondary gap is **compositional layer completeness** — 4 annotation categories that declare layer constraints but don't generate structural IR.

---

## DSL File Audit: Modeling Quality by Layer

### Layer Assessment Summary

| Layer | Files | Lines | Modeling quality | Key finding |
|-------|-------|-------|-----------------|-------------|
| `std/` (primitives) | 24 | ~2,800 | **70% — rich vocabulary, systematic holes** | Types are well-designed but not used by consumers |
| `extdeps/` (external models) | 35 | ~3,500 | **85% — genuinely strong** | Real behavioral modeling, honest unknowns, typed errors |
| `services/` (transport) | 14 | ~1,800 | **50% — imports types, doesn't use them** | Dead imports universal; `String` where refinements exist |
| `tools/` (workflows) | 13 | ~900 | **60% — range from production to skeleton** | makegen excellent, testgen/deps broken |
| `workflows/` (pipelines) | 10 | ~500 | **20% — topology only** | Almost all stage bodies empty |

### std/ Primitives: What's Strong

The type vocabulary in `std/types.dag` (858 lines) is genuinely well-designed:

- **Refinement types used extensively**: `CommitSha` (`@pattern`), `HttpStatus` (`@range`), `Email` (`@pattern`), `GistId` (`@format`), `Port` (`@range(1,65535)`), branded IDs (`IntentId`, `IssueId`, `RunKey`, etc.)
- **Sum types with payloads**: `CredentialFlow`, `AuthScheme`, `SignalType`, `ArtifactPayload`, `FailureClass`, `AuditAction`
- **SDLC domain model comprehensive**: two-phase commit, distributed leases, CAS counters, retry budget, audit trail
- **State machine module (`state_machines.dag`)**: cleanest module — proper `TransitionResult` sum type, ordinal-based validation, no string dispatch
- **Fermi/fidelity vocabulary**: ordinal algebra, depth comparison, budget checking — correct compositional layering
- **Unicode width tables**: data-oriented design with block-range lookup — correct approach
- **Filesystem classification**: `ReadableEntry = FileEntry where is_text_readable` — correct `@where` refinement

### std/ Primitives: Systematic Holes

**1. Timestamp fields as raw `String` (~20 sites)**

`Timestamp` is defined as a refinement type in `types.dag` but not used:

| Field | File | Should be |
|-------|------|-----------|
| `created_at: String` | types.dag (Artifact, ArtifactMarker, StageOutcome) | `Timestamp` |
| `updated_at: String` | types.dag (StageOutcome) | `Timestamp` |
| `acquired_at: String` | types.dag (ClaimLease) | `Timestamp` |
| `expires_at: String` | types.dag (ClaimLease, AccessToken, Signal) | `Timestamp` or `Timestamp?` |
| `produced_at: String` | types.dag (Signal) | `Timestamp` |
| `consumed_at: String?` | types.dag (Signal) | `Timestamp?` |
| `timestamp: String` | types.dag (AuditEntry) | `Timestamp` |
| `timestamp: String` | resources.dag (Clock.now output) | `Timestamp` |

**2. Stringly-typed enumerations (~15 sites)**

Known closed sets of values using `String` instead of sum types:

| Field | Known values | Should be |
|-------|-------------|-----------|
| `TopologyNode.kind` | `"pure" \| "transport" \| "subdag" \| "env"` | Sum type |
| `DocSource.kind` | `"template" \| "generated" \| "static"` | Sum type |
| `DesignFinding.severity` | `"blocking" \| "suggestion" \| "info"` | Sum type |
| `DesignOutput.source` | `"llm" \| "human" \| "policy"` | Sum type |
| `IssueBinding.provider` | `"github" \| "linear" \| "local-blob"` | Sum type |
| `PipelineArtifact.kind` | `"design" \| "review" \| "plan"` | `ArtifactType` (already exists!) |
| `RetryPolicy.retry_on` | `["rate_limit", "timeout", "http_429"]` | `List<RetryTrigger>` enum |

**3. `ContentHash` brand not applied (3 sites)**

`ContentHash = NonEmptyStr where brand("ContentHash")` is defined but `StageRunKey.input_hash`, `Artifact.content_hash`, `ArtifactMarker.content_hash` all use `NonEmptyStr`.

**4. Unit inconsistency: seconds vs milliseconds**

`Duration` in `types.dag` is milliseconds. `LeaseConfig`, `QueueSemantics`, `RateLimit` use seconds. `fermi_timeouts` uses milliseconds. No unit-branded types distinguish these.

**5. Duplicate type definitions**

- `EntryKind` + `SymlinkTarget`: defined in both `types.dag` and `filesystem.dag`
- `RetryPolicy`: defined in both `types.dag` (with backoff fields) and `rate_limit.dag` (with `BackoffStrategy` sum type) — **different structures, same name**

**6. Missing types that should exist**

| Missing type | Where needed |
|-------------|-------------|
| `SeverityLevel` enum | `DesignFinding.severity`, `EdgeCase.severity` |
| `DataSource` enum | `DesignOutput.source`, `DesignReviewOutput.source` |
| `RetryTrigger` enum | `RetryPolicy.retry_on` |
| `LanguageId` branded type | `Language.id` |
| `GcpRegion` branded type | `GcpProviderConfig.region` |
| `VirtualEnvSetup` | Environment variable mocking |
| Canonical error wrapper | Spanning all provider error shapes |
| C/MIPS/Dag language defs | `CodegenBackend` has 4 variants, `languages.dag` has 2 |

**7. Stubs that look like features (violates PR-gate)**

| Stub | Location | Issue |
|------|----------|-------|
| `retry<Op>` pattern | `patterns.dag` | Body is empty — no retry logic wired |
| `approval_yield` pattern | `patterns.dag` | Comment-only, no DSL declaration |
| `check_iam_binding` | `patterns.dag` | Returns `false` always |
| `add_iam_binding` | `patterns.dag` | Identity passthrough |
| `AuthContext.acquire` | `resources.dag` | Empty body |
| `Network` resource | `resources.dag` | No capabilities declared |
| `Filesystem.read` takes `String` | `resources.dag` | Comment says `TextFilePath`, type says `String` |
| `fs.stat` called but undeclared | `patterns.dag` | Broken reference |

### services/: The Type Discipline Gap

The central finding: **services import extdeps domain types but only use them in `response {}` blocks, not in `input {}` or `output {}` blocks.** The typed model exists upstream but dissolves at the service boundary.

**Dead imports (every service file)**

| Service | Dead imports |
|---------|-------------|
| `git.dag` | `GitCommit`, `GitRemote`, `GitMergeResult`, `DiffLine`, `GitFileStatus` |
| `cargo.dag` | `CargoManifest`, `CargoDependency`, `CargoProfile`, `CargoFeature`, `RustChannel`, `CargoCommand`, `RustEdition` (all 7) |
| `github/pull_request.dag` | `PrState`, `PrBranchRef`, `PrMergeStrategy`, `ReviewState`, `PrFileStatus`, `CheckStatus`, `CheckConclusion` |
| `llm/anthropic.dag` | `ThinkingConfig`, `AnthropicApiVersion`, `SystemPrompt`, `ContentBlock`, `TokenUsage`, `StopReason` |
| `llm/openai.dag` | `OpenAiModel`, `OpenAiModelSpec`, `ResponseFormat`, `ToolChoice`, `OpenAiFinishReason`, `FunctionCall`, `ToolCall`, `TokenUsage`, `StopReason` |

**Concrete type downgrades (worst cases)**

| Service | Field | Current type | Should be |
|---------|-------|-------------|-----------|
| `anthropic.dag` | `messages` input | `Json` | `List<LlmMessage>` |
| `anthropic.dag` | `model` input | `String` | `AnthropicModel` |
| `anthropic.dag` | `stop_reason` output | `String` | `StopReason` |
| `openai.dag` | `messages` input | `Json` | `List<LlmMessage>` |
| `openai.dag` | `model` input | `String` | `OpenAiModel` |
| `pull_request.dag` | `files` output | `Json` | `List<PrFile>` |
| `pull_request.dag` | `reviews` output | `Json` | `List<Review>` |
| `pull_request.dag` | `state` output | `String` | `PrState` |
| `pull_request.dag` | `head_sha` output | `String` | `CommitSha` |
| `gcp/iam.dag` | `bindings` output | `Json` | `List<GcpBinding>` |
| `cargo.dag` | All outputs | `stdout: String, stderr: String` | Structured types |

**Missing behavioral properties**

| Service | Operation | Missing |
|---------|-----------|---------|
| `github/issues.dag` | `get`, `discover`, `list_events` | `readonly` |
| `github/pull_request.dag` | `Get`, `ListFiles`, `ListReviews` | `readonly` |
| `github/pull_request.dag` | `Merge` | `idempotent` |
| `github/issues.dag` | `close`, `set_labels` | `idempotent` |

**Missing `auth_input`**

`gist.dag` has `auth_input: auth_token`. `issues.dag`, `pull_request.dag`, `anthropic.dag`, `openai.dag` all declare `auth: BearerToken` but no `auth_input`.

### extdeps/: The Strong Layer

This is the best-modeled layer. Key strengths:

- **`OperationBehavior` schema uniformly applied**: every API-surface file has per-operation behavioral declarations with `side_effects`, `idempotent`, `determinism`, `failure_modes`, `edge_cases`, `unknowns`
- **Honest epistemic modeling**: `unknowns` fields populated with real operational uncertainty (e.g., "Exact retry-after header behavior under sustained load", "Whether deleted labels appear in timeline", "Registry-side indexing delay")
- **Correct idempotency analysis**: push is `recoverable: true` but `retry_safe: false` (requires rebase first); delete-gist is `idempotent: true` with `idempotency_keys: ["gist_id"]`; LLM calls are `NonDeterministic`
- **Sum types where domain requires**: `GitRef` (3 variants), `ContentBlock` (3 variants with payloads), `CloudAuthScheme` (4 variants with parameters), `SecretEncryption` (2 variants)
- **Workload Identity Federation modeled explicitly** in `cloud/gcp/core.dag` — pool, provider, attribute mapping
- **Anthropic 529 status code** modeled — provider-specific knowledge

### tools/ and workflows/: The Execution Gap

**Production-quality tool files**: `makegen.dag` (35 DSL fn items, 1 extern), `codegen.dag` (fully DSL), `gist.dag` (explicit `uses` declarations, typed outputs)

**Broken tool files**:
- `testgen.dag`: calls undefined `generate()` method — will not compile
- `deps.dag`: calls undeclared `parse_deps_toml`, `shell_check`, `shell_exec`

**Workflow files are topology-only**: `ci.dag` has 12 stages, all empty. `gist.dag` has 10/12 stages empty. `pragma.dag` has 7/9 stages empty. Only `makegen.dag` has real stage bodies with data flow.

**Missing `uses` declarations**: `makegen`, `pragma`, `build` all call `content_upsert` (requires `Filesystem(mode: ReadWrite)`) but don't declare `uses fs:`.

---

## The Overall Picture

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

This is the same pattern as the presence algebra gap, just at a different level: **the system has the right abstractions but doesn't enforce their use.** The compiler accepts `input { model: String }` when `AnthropicModel` is in scope and imported. No lint, no warning, no error.

---

## Appendix A: Silent Coercion Sites (Presence Axis)

Every location where `Value::Skipped` is silently treated as a concrete value:

| File | Function | Coercion |
|------|----------|----------|
| `daglang-lower/src/eval.rs` | `value_to_string` | `Skipped → ""` |
| `daglang-lower/src/eval.rs` | `value_truthy` | `Skipped → false` |
| `daglang-lower/src/eval.rs` | `values_equal` | `Skipped == Unit → true` |
| `daglang-lower/src/eval.rs` | `field_access` | `Skipped.field → Unit` |
| `daglang-lower/src/eval.rs` | `sort_key` | `Skipped → "skipped"` |
| `core/exec/src/execute/mod.rs` | `collect_fan_in` | `Skipped → dropped from list` |
| `core/exec/src/pattern_op.rs` | `list_values` | `Skipped → vec![]` |

## Appendix B: Evaluator Silent Behaviors (Runtime Strictness)

Operations that silently produce defaults instead of erroring:

| Operation | Silent behavior | Should be |
|-----------|----------------|-----------|
| `div/mod by zero` | Returns `0` | Error |
| `map/filter/flat_map` missing lambda | Returns list unchanged | Error |
| `sum` on non-Int elements | Filters silently | Error |
| `join` non-Str separator | Defaults to `","` | Error |
| `contains` no needle | Returns `false` | Error |
| `sort_by` key eval error | Uses `""` | Propagate error |
| `for` on scalar | Wraps in `[scalar]` | Error or require explicit wrap |
| `first/last` on empty list | Returns `Unit` | Return `Optional` (cardinality-aware) |
| Uppercase unbound ident | `Value::Str(name)` heuristic | Validate against known variants |
| Field access on JSON object, missing field | `Json(Null)` | Error (consistent with Map behavior) |
| `if` without `else` | `Unit` | Require `else` in expression position, or return `Optional` |
| Match non-exhaustive | Runtime error | Static exhaustiveness check |

## Appendix C: Inconsistencies (Same Concept, Different Enforcement)

| Concept | Strict path | Lenient path |
|---------|------------|--------------|
| Field access on missing | `Map.missing → error` | `Json.missing → Null` |
| Skipped in field access | `GetFieldOp` (resolve) → error | `field_access` (eval) → `Unit` |
| `when` guard type | Pipeline → type-checked as Bool | Node → unchecked |
| Unresolved targets | Strict mode → error | Relaxed mode (default!) → silent |
| Callable output wiring | `fn_body: None` → validated | `fn_body: Some` → exempt |
