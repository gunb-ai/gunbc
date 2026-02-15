# V2 Contracts Design

**Status**: Draft — January 2026
**Scope**: First-principles design for the modeling layer's type system.
**Motivation**: See [`bl1-retrospective.md`](./bl1-retrospective.md) for the
problem analysis that led here.

---

## Glossary

| Term | Definition |
|------|------------|
| **Understanding** | A tool/system model boundary; owns pattern composition and registry identity |
| **Behavior** | An instantiated node produced by pattern instantiation — **not** an author-authored flat list item |
| **Pattern** | A sub-DAG template defining a causal sequence of operations with typed interfaces |
| **PatternDef** | The template itself (nodes, edges, port types, guards) |
| **PatternInstance** | A tool's binding of a PatternDef to concrete implementations, identified by `PatternInstanceId` |
| **\*Spec** (e.g., `UpsertSpec`) | The bindings struct for a specific pattern kind |
| **ContractNode** | The V2 primitive: a node declaring typed Requires/Provides/Imports/Exports/Claims |
| **SubDag** | The output of pattern instantiation: nodes + typed edges + output handle |
| **GraphIR** | The execution-level DAG (Block → Block with typed edges, ports, waves) |
| **`gunbai-dag`** | Generic DAG algorithms (topo sort, cycle detection, wave scheduling) |
| **Lowering** | Translating modeling-layer pattern specs into execution-layer GraphIR |
| **Lane A/B/C** | Extension lanes: stable core enums / declared typed extensions / doc-only strings |
| **Modeling pattern** | Compile-time semantic template (V2) — not to be confused with runtime execution wrappers |

---

## Terminology Warning

In V1, "Behavior" means an author-defined list entry in a flat
`&[Behavior]` array. In V2, "Behavior" refers to a node produced by pattern
instantiation. To avoid confusion, this document uses **ContractNode** when
referring to the V2 node primitive. When "behavior" appears unqualified, it
means the V2 sense (instantiated node).

---

## 1. Thesis

A modeling system for tool behaviors must make invalid states
unrepresentable. Specifically:

- Every behavioral pattern is a **causal DAG** — a directed graph of
  operations with typed data flow between them.
- Tools don't "have" patterns. They **instantiate** pattern templates by
  binding concrete implementations to typed node slots.
- **Behaviors are not authored as a flat list; they are the nodes produced
  by pattern instantiation.** Authors supply pattern bindings, not behavior
  lists.
- Semantic roles (phases, failure kinds, output characteristics) are
  **enums**, not strings. Identifiers are **validated newtypes**.
- Every property claim carries its own **verification strategy**. Unverified
  assertions are structurally impossible to construct silently.

### The V2 primitive

The core primitive is a **`ContractNode`**: a node in a causal graph that
declares typed `Requires`/`Provides`, typed `Imports`/`Exports`, typed
`Claims`, and verification bindings.

- **Patterns** are templates over `ContractNode`s
- **Tools** are compositions of pattern instances (each binding concrete
  implementations to a template's `ContractNode` slots)
- **Understandings** are registries packaging these compositions

A `PatternDef` is a *template* that can be instantiated into a sub-DAG.
A `*Spec` (e.g., `UpsertSpec`) is the set of bindings from tool-specific
implementations into the template's slots. A `PatternInstance` is one
instantiation of a template, identified by `PatternInstanceId`.

### V2 deliverables

- [ ] Replace six semantic string channels with typed equivalents
- [ ] Upsert becomes structural sub-DAG (no optional phase tags)
- [ ] Lane B registry exists + validated (custom codes declared, unique, namespaced)
- [ ] Generators consume typed semantics; no prefix parsing remains
- [ ] Property claims require verification binding
- [ ] Typed IDs with construction choke points (`UnderstandingId`, `BehaviorId`)
- [ ] Conditional execution semantics specified for pattern sub-DAGs
- [ ] Pattern instance identity supports multiplicity

### Non-goals for V2

- Not modeling watch/lock/cache/retry patterns until two tools need them
- Not fully generalizing patterns to arbitrary DAG templates on day 1 —
  upsert first, generalize when a second pattern earns formalization
- Not removing string representations for IDs in user-facing surfaces
  (CLI output, documentation anchors, test names still display strings)
- Not building a runtime executor — V2 is the modeling/compiler layer only;
  `gunbai-dag` is the executor and is not being redesigned
- Not preserving BL1 API compatibility
- Not inferring semantics from tool outputs at runtime
- Not making the runtime pattern-aware — `gunbai-dag` sees only
  dependencies, claims, and scheduling

---

## 2. Design Principles

### P1: Patterns Before Tools

Define the behavioral vocabulary (patterns) first. Tools are compositions of
patterns. A tool that doesn't map to known patterns is either exposing a new
pattern (which must be formalized) or is incorrectly modeled.

**Enforcement mechanism**: every behavior must declare a `BehaviorRole`.
Freeform `BehaviorId` is only allowed for `BehaviorRole::Custom(...)` and
must be namespaced. This means every new behavior is either:

- a new role in an existing pattern, or
- a new pattern proposal (requires 2+ tool use cases per P3), or
- explicitly custom and auditable

### P2: No Freeform Strings for Semantics — Extension Lanes

Every semantic field is a typed enum or structured reference. Three lanes
govern extensibility:

- **Lane A (stable core)**: `FailureKind::{NotFound, PermissionDenied, …}`.
  Breaking change. Rare and deliberate.
- **Lane B (declared extensions)**: `FailureKind::Custom(CustomFailureCode)`
  where `CustomFailureCode` is a **typed, namespaced identifier** — a
  declared constant or macro-produced symbol, not arbitrary `&str`. Gives
  auditability, metrics aggregation, and typo prevention without requiring
  core crate changes.
- **Lane C (doc-only)**: `EdgeCase(String)`. Pure documentation, no semantic
  load.

**Semantic boundary rule**: any field consumed by generators, validators,
registry lookup, dep resolution, or test naming is semantic and must be
typed (Lane A or Lane B). Fields consumed only by documentation rendering
are Lane C. This boundary is normative and not subject to case-by-case
review.

**Lane B registry mechanism**: `CustomFailureCode` (and `CustomBehaviorId`,
`CustomDependencyTarget`, etc.) are constructed via `declare_*!()` macros
that register the code in a central inventory. A generated test validates:

- canonical formatting (namespace/name)
- uniqueness across all declarations
- owner tool ID prefix rules

Construction outside the macro is banned (enforced by private inner field).
One way to create, many ways to use.

**Extension lane doctrine**: every semantic axis may have Lane A (stable
core variants), Lane B (typed custom code variant), and Lane C (doc-only).
Currently Lane B is implemented for failure kinds, behavior roles, and
dependency targets. The same mechanism applies to any future semantic axis
(output semantics, pattern IDs, etc.) without ad-hoc exceptions.

**Resolving the stability/extensibility tension**: Lane B is the incubation
lane. A tool can declare a typed, namespaced extension code without promoting
it to a core enum variant. Promotion to Lane A requires 2+ tools using it.
This means semantics are typed from day one, but stabilization requires
evidence.

### P3: No Speculative Patterns — And No Speculative Types

Every pattern must have at least two concrete tool use cases before
formalization. Patterns without use cases are deleted.

**Corollary: no speculative types.** If a type exists primarily to support a
pattern, it must be created in the same PR as at least two tool
instantiations of that pattern and the generated validations/tests. This
prevents "type-first" speculation even inside a "pattern-first" story.

**Pattern incubation lanes** (analogous to P2's extension lanes):

- **Pattern Lane A (stable core)**: included in `Patterns { upsert, … }`,
  supported by generators/validators, has `PatternUse<T>` field.
- **Pattern Lane B (declared extensions)**: `CustomPatternId` — typed and
  structurally representable (still a sub-DAG with typed nodes/edges), but
  not yet a named field in the core `Patterns` struct. Declared via
  `declare_pattern!()`. Promotion to Lane A requires 2+ tools.
- **Pattern Lane C (doc-only)**: "this tool exhibits X shape" — noted in
  documentation, not modeled.

This resolves the P1/P3 tension: the *first* tool needing a new pattern
declares it in Lane B (typed, structural, auditable). The *second* tool
triggers promotion to Lane A. You can always add the first tool; you can't
bloat the core without evidence.

### P4: Every Property Claim Carries Its Verification

The claim itself carries its verification strategy:

```rust
struct PropertyClaim {
    property: Property,
    verified_by: Verification,
}

enum Verification {
    /// A test is generated from the claim.
    GeneratedTest(GeneratedTestSpec),
    /// An integration harness validates the claim.
    Harness(HarnessSpec),
    /// Property is a logical consequence of other verified properties.
    /// Machine-checkable: CI verifies the cited properties exist and are
    /// themselves verified, and the rule is in the approved derivation set.
    Derived {
        from: &'static [Property],
        rule: DerivationRuleId,
    },
    /// Last resort. Valid only when the property concerns external systems
    /// we cannot mock and testing is infeasible (not just inconvenient).
    ManualJustification {
        reason: &'static str,
        allowlist_entry: AllowlistRef,
    },
}
```

`Derived` replaces the previous practice of writing "Deterministic implies
idempotent" as a ManualJustification reason string. Derivations are
machine-checkable and don't require allowlist entries. CI enforces that:

- The `from` properties exist and are themselves verified
- The `rule` is in the approved derivation rule set

`ManualJustification` requires an explicit allowlist entry and fails CI
without one. "Inconvenient to test" is not a valid reason. Allowlist entries
must have an owner, an expiry date or version milestone, and a tracking issue
ID. This prevents temporary justifications from becoming permanent folklore.

**Default verification strategy per property** (prevents "verification theater"):

| Property | Default verification | Notes |
|---|---|---|
| `ReadOnly` | Harness or generated "no diff" test | Needs defined "world snapshot" boundary |
| `Deterministic` | Generated test (N runs, identical output) | Requires stable input fixture |
| `Idempotent` | Generated test (run twice, no extra diff) | May be `Derived` from Deterministic via approved rule |
| `WritesWorld` | Harness (measures side effects) | Generated tests can't prove it for black-box tools |
| Prerequisites | Structural dependency edges | Not a runtime test — modeled as `DependsOn` |

### P5: Identity vs Semantics

**Semantic roles** (phases, failure kinds, behavior roles) are enums.
**Identifiers** (registry keys, documentation anchors) are validated newtypes
with canonical formatting. Construction goes through a single choke point:

```rust
pub struct UnderstandingId(&'static str);

impl UnderstandingId {
    /// Const-validated construction for static definitions.
    pub const fn new_const(s: &'static str) -> Self {
        // validates: allowed charset, slash-separated segments, non-empty
        Self(s)
    }
}

/// Convenience macro with compile-time validation.
macro_rules! understanding_id {
    ($s:literal) => {{
        const ID: UnderstandingId = UnderstandingId::new_const($s);
        ID
    }};
}
```

- `UnderstandingId` — newtype, const-constructible with validation
- `BehaviorId` — **derived from role** for patterned behaviors (no
  author-supplied strings), author-supplied only for `Custom`.
  **Locality constraint**: a `BehaviorId` is not constructible without its
  parent `UnderstandingId`. No floating behaviors.

**Canonical ID derivation scheme** for patterned behaviors:

| Component | Source | Example |
|---|---|---|
| `UnderstandingId` | Author-supplied via `understanding_id!()` | `"tool/zstd"` |
| `PatternInstanceId` | `(UnderstandingId, PatternKind, InstanceSlug)` | `("tool/zstd", Upsert, "binary")` |
| `BehaviorId` | `(UnderstandingId, PatternInstanceId, Role)` | `"tool/zstd/upsert:binary/check"` |

For patterned behaviors, there is no `BehaviorId` field in author code at
all. The ID is derived from `UnderstandingId` + `PatternInstanceId` + `Role`
(e.g., `UpsertPhase::Check`). The display representation (`"tool/zstd/check"`)
is generated by `Display`, not hand-written. Display strings are stable under
enum variant renames because each role has an explicit string mapping (not
derived from Rust variant names).

**Dependency scoping rules**: dependencies may scope to:

- A pattern instance (`PatternInstanceId`)
- A role within an instance (`BehaviorRole`)
- A custom behavior (`CustomBehaviorId`)

Dependencies must **not** scope to derived string IDs. This keeps scoping
semantic and stable under refactoring.

### P6: Resource Lifecycle as First-Class Concept

Resources declare their lifecycle kind:

- **Ephemeral** — created and destroyed within a scope
- **Persistent** — survives across invocations
- **Borrowed** — referenced but not owned

### P7: Model the Axis, Not the Instances

Typed semantic enums model **dimensions of meaning**, not individual values:

**Failure taxonomy**:

```rust
enum FailureKind {
    NotFound,
    AlreadyExists,
    PermissionDenied,
    NetworkUnavailable,
    VersionMismatch,
    Timeout,
    Custom(CustomFailureCode), // Lane B
}

/// Derived from FailureKind — generators/executors use this for
/// retry/backoff decisions without matching on every variant.
enum FailureClass {
    Transient,       // retryable (NetworkUnavailable, Timeout)
    Permanent,       // not retryable (PermissionDenied, VersionMismatch)
    Indeterminate,   // Custom codes — retryability declared per-code
}
```

**Output semantics**:

```rust
enum OutputSemantics {
    PureSignal,                        // success/failure only
    CreatesResource,                   // produces a handle
    UpdatesResource,                   // modifies existing
    NoOpWhen(NoOpCondition),           // e.g. HTTP 304
    CoercesSuccess(SuccessCoercion),   // reinterprets non-zero exit
}

enum NoOpCondition { HttpNotModified, AlreadyUpToDate, CacheHit }
```

Small sub-enums keep the top-level stable while allowing domain-specific
extension without string drift.

---

## 3. Patterns Are Causal DAGs

### The three-level DAG

The system has three natural levels of causal structure:

| Level | What it captures | Example |
|---|---|---|
| **Understanding → Understanding** | Which tools depend on which tools | git depends on network; tectonic depends on LaTeX packages |
| **Behavior → Behavior** | Which operations within a tool form causal sequences | Check → Create → Resolve within upsert |
| **Block → Block** | Which execution steps connect via typed data flow | The GraphIR execution graph |

Level 1 exists today (`depends_on` + `ancestors()`/`descendants()`).
Level 3 exists today (GraphIR + `gunbai-dag`).
**Level 2 is missing.** Behaviors within an understanding are a flat
`&[Behavior]` with no edges. Pattern annotations (`UpsertPhase`) tag
behaviors but don't create causal edges between them.

V2 fills Level 2: patterns are sub-DAG templates that create typed edges
between behaviors. The `Pattern` trait's `instantiate()` returns a `SubDag`
— behaviors connected by typed data flow — which the lowering phase unrolls
into Level 3's GraphIR.

### The core insight

If a pattern defines a causal sequence (A must happen before B, B's output
feeds C), then a pattern *is* a directed acyclic graph. Modeling it as
anything less — an enum tag, an annotation, a phase field — loses the
causal structure.

### Upsert as a sub-DAG

Three nodes with typed data flow:

```
Check   →  Exports: ResourceState (Exists | Missing | Stale)
Create  →  Imports: ResourceState (guarded: Missing only), Exports: ResourceRef
Resolve →  Imports: ResourceState | ResourceRef, Exports: ResolvedHandle
```

Check's output determines whether Create executes. Resolve runs regardless.
This is a causal graph, not three behaviors with a shared enum tag.

#### Conditional execution semantics

Pattern sub-DAGs support **guarded nodes**: a node can declare a guard on an
input port that determines whether the node executes or is skipped.

```rust
struct PortGuard<T> {
    /// The node only executes when this predicate holds on its input.
    /// When the guard fails, the node is skipped and produces `Skipped`.
    predicate: GuardPredicate<T>,
}

enum GuardPredicate<T> {
    /// Execute when input equals this value.
    Equals(T),
    /// Execute when input matches this variant.
    MatchesVariant(/* discriminant */),
    /// Always execute (no guard).
    Always,
}

/// A guarded node's output is wrapped to distinguish execution from skip.
enum GuardedOutput<T> {
    Produced(T),
    Skipped,
}
```

For upsert specifically:

- **Check**: no guard (`Always`) — always runs. Exports `ResourceState`.
- **Create**: guard `Equals(ResourceState::Missing)` — skipped when resource
  exists. Exports `GuardedOutput<ResourceRef>`.
- **Resolve**: no guard — always runs. Imports
  `(ResourceState, GuardedOutput<ResourceRef>)` and produces
  `ResolvedHandle`.

This is uniform across all patterns. Any node in any pattern sub-DAG can
declare a guard. The lowering phase translates guards into GraphIR's
execution semantics (skip node + propagate `Skipped` downstream). Contract
tests verify that Create is un-runnable in the Exists case, not merely that
it "fails."

### The Pattern trait

```rust
/// A pattern is a sub-DAG template with typed node slots.
trait Pattern {
    /// What the pattern produces when fully executed.
    type Output;
    /// Concrete bindings for this pattern's node slots.
    type Bindings;

    /// Bind concrete implementations to the template's slots.
    /// Returns a sub-DAG that can be lowered into gunbai-dag.
    fn instantiate(bindings: Self::Bindings) -> SubDag<Self::Output>;
}

/// SubDag carries the graph AND a typed output handle.
struct SubDag<O> {
    nodes: Vec<NodeDef>,
    edges: Vec<EdgeDef>,
    /// Typed reference to the terminal node's output port.
    /// Erased during lowering into GraphIR port references.
    output: OutputRef<O>,
}

/// Upsert: Check → Create (guarded) → Resolve
struct UpsertPattern;

impl Pattern for UpsertPattern {
    type Output = ResolvedHandle;
    type Bindings = UpsertBindings;

    fn instantiate(bindings: UpsertBindings) -> SubDag<ResolvedHandle> {
        // Constructs a 3-node sub-DAG with:
        // - Check: Always guard, exports ResourceState
        // - Create: Equals(Missing) guard, exports GuardedOutput<ResourceRef>
        // - Resolve: Always guard, imports both, exports ResolvedHandle
        // Binding signature mismatch = compile error.
    }
}
```

`UpsertBindings` holds function pointers (or closures) to the concrete
implementations. Signature changes break at compile time.

### Examples of candidate future patterns (not shipping in V2)

Any behavioral pattern with sequencing, conditional execution, or data flow
between steps is a causal graph. These are candidates identified from
existing tools — **not** planned V2 patterns. They would require 2+ tool
use cases and a formal proposal per P3 before formalization:

| Candidate pattern | Causal structure | Known use cases |
|---|---|---|
| Generator pipeline | Dependency waves with resource locks | depgen, invariantsgen |
| Two-phase auth | Preflight (parallel) → Interactive → Config → Fetch | login |
| Source → Plan → Apply | Three sub-DAGs; state propagation between phases | infra apply |

---

## 4. Pattern Composition

### Pattern instance identity and multiplicity

A tool may need multiple instances of the same pattern (e.g., a tool that
manages two different resources, each with its own upsert). Pattern instances
are explicitly identified:

```rust
/// Identifies one instantiation of a pattern within a tool.
struct PatternInstanceId {
    understanding: UnderstandingId,
    kind: PatternKind,
    instance: InstanceSlug, // validated newtype, e.g. "binary", "config"
}

/// For single-instance cases, a default slug is implied.
impl PatternInstanceId {
    fn default_for(understanding: UnderstandingId, kind: PatternKind) -> Self {
        Self { understanding, kind, instance: InstanceSlug::DEFAULT }
    }
}
```

This means:

- `BehaviorId` is derived from `(UnderstandingId, PatternInstanceId, Role)`
  — no collisions even with multiple instances.
- `CompositionSpec::Chained { sequence: Vec<PatternInstanceId> }` references
  something stable and typed.
- Single-instance tools use a default slug and don't pay verbosity cost.

### Declaring patterns

A tool understanding is a composition of pattern specs, not a bag of
behaviors with optional tags:

```rust
enum PatternUse<T> { NotApplicable, Applicable(T) }

struct Patterns {
    upsert: PatternUse<Vec<UpsertInstanceSpec>>,
    lifecycle: PatternUse<Vec<LifecycleInstanceSpec>>,
    capabilities: Vec<CapabilitySpec>,   // Lane B: what the tool *does* for others
    composition: CompositionSpec,
}

struct UpsertInstanceSpec {
    id: InstanceSlug,
    resource: ResourceKind,
    bindings: UpsertBindings,
}
```

`NotApplicable` is a **contract assertion** — "I have reviewed this pattern
and determined it does not apply." `Option<T>` means "I forgot about this";
`PatternUse<T>` means "I decided about this."

When a new pattern is added to the registry, a generated test (or compile
error) forces every tool understanding to address it — either
`Applicable(...)` with a complete spec, or `NotApplicable`. This is
mechanical but intentional: explicit opt-out is the structural guarantee.

### Composition semantics

When a tool implements multiple patterns:

```rust
enum CompositionSpec {
    /// Patterns are independent sub-DAGs (no shared nodes)
    Independent,
    /// One pattern's output feeds another's input (non-empty, acyclic)
    Chained { sequence: Vec<PatternInstanceId> },
}
```

**Invariants**:
- `Chained.sequence` must be non-empty and acyclic

~~`Shared { bindings: Vec<NodeBinding> }` was considered but cut from V2.~~
Shared node ownership requires a constraint solver (guard conflicts between
patterns claiming the same node). If two patterns truly share a node, they
are likely a single, larger pattern. Revisit only with concrete evidence.

### "Install" collapses into Upsert

**Criterion**: if the behavior is "ensure X exists and return a handle to X
(or confirmation)," it is upsert. "Install" is an upsert where Create has
sub-steps (download, verify, extract).

In V2, Create is modeled as a single behavior node even if the underlying
implementation is multi-step. Pattern nesting (Create itself being a
sub-DAG) is a future extension if multiple tools require it. There is no
separate "install pattern."

---

## 5. Registry Architecture

### Node kinds and edge kinds

A single registry for uniform lookup/docs/introspection, but with
distinguished node and edge kinds:

- **Node kinds**: `Node::Understanding(ToolUnderstanding)` vs
  `Node::Pattern(PatternDef)`
- **Edge kinds**:
  - `Edge::DependsOn` — real ordering, participates in depgen
  - `Edge::ImplementsPattern` — meta, does *not* participate in dep resolution
  - `Edge::ValidatedBy` — test generation linkage

**Hard rule**: only `Edge::DependsOn` participates in dependency resolution
and topological ordering. All other edge kinds are ignored by depgen,
execution DAG composition, and install ordering. This prevents generators
from accidentally following meta edges.

---

## 6. Compiler Architecture

The predecessor Go system overlaid a DAG contract (`NodeContract`) directly
on executable code (`func() error`). That worked because the DAG constrained
real functions. This system introduced a genuinely new concept — a modeling
layer that describes external tools without executing them — but built it as
flat documentation rather than as a causal graph. The compiler architecture
bridges this: the modeling layer produces causal structure (sub-DAGs), and the
lowering phase translates that structure into the execution layer's IR.

The V2 system is a **compiler** with three layers:

```
┌──────────────────────────────────────────────────┐
│  Modeling layer (front-end)                       │
│  Understandings + Pattern specs + sub-DAG templates│
├──────────────────────────────────────────────────┤
│  Lowering (validation + unrolling)                │
│  Pattern templates → flat nodes + typed edges     │
│  Guards → skip semantics + Skipped propagation    │
├──────────────────────────────────────────────────┤
│  Execution layer (assembly / gunbai-dag)          │
│  Dependencies, resource claims, wave execution    │
└──────────────────────────────────────────────────┘
```

- **Modeling** validates pattern specs against contracts
- **Lowering** unrolls pattern templates into flat `gunbai-dag` nodes/edges;
  incomplete or inconsistent specs fail here with typed errors; guards are
  translated into skip semantics
- **Execution** understands only dependencies, claims, and scheduling

`gunbai-dag` must not understand patterns. It is the runtime. The modeling
layer is the compiler.

### Enforcement levels

To be precise about what "compile-time guarantee" means in this context:

- **Per-node invariants** (e.g., "UpsertSpec must have all three phases") are
  enforced **structurally at construction time** — you cannot represent an
  invalid shape. This is true Rust compile-time enforcement.
- **Cross-registry invariants** (e.g., "all CustomFailureCodes are unique",
  "all tools address all patterns") are enforced via **generated validation
  tests**. These are still structural — no string parsing — but they run at
  test time, not compile time.

Both are "structural." The distinction is binding time: construction vs
test suite.

---

## 7. Structural Guarantees

These states must be **unrepresentable**:

1. **No upsert without complete phases** — `UpsertSpec` requires Check +
   Create + Resolve at construction time. Partial participation is a type
   error. The old `Option<UpsertPhase>` is banned.

2. **No set scoping without `SetSpec`** — all scoping over behaviors,
   properties, or dependencies goes through `SetSpec<T>`. No raw slices.

3. **No semantic roles as strings** — behavior roles, dependency targets,
   failure kinds are enums. Identifiers are validated newtypes. The `Custom`
   extension lane uses declared codes, not arbitrary `&str`.

4. **No convention-based protocols** — any protocol that relies on string
   prefix parsing (e.g., `"success:http_304"`) must become typed variants.
   Conventions drift; contracts don't.

5. **No behaviors without pattern provenance** — behaviors are produced by
   pattern instantiation. There is no `Vec<Behavior>` to populate manually.
   Custom (non-patterned) behaviors use `BehaviorRole::Custom` with a
   declared Lane B code.

### V2 acceptance criteria

V2 is done when these pass:

- [CT] No behavior exists without a `BehaviorRole` (patterned derives role; custom requires declared role)
- [CT] Any `PatternUse::Applicable` is structurally complete (e.g., Upsert has all three phases)
- [GT] All `CustomFailureCode`s (and other Lane B codes) are declared in one registry and validated for uniqueness + formatting
- [GT] All tools address all core patterns (`PatternUse` — `Applicable` or `NotApplicable`)
- [CT] No generator parses string prefixes for semantics
- [GT] All properties have `verified_by`; manual justifications are allowlisted with owner + expiry + tracking issue
- [CT] `BehaviorId` for patterned behaviors is derived, not authored
- [CT] `UnderstandingId` construction goes through validated choke point
- [CT] Conditional execution (guards) is represented in SubDag and lowered correctly
- [GT] `Verification::Derived` claims cite approved rules and verified source properties

**Legend**: CT = construction-time (cannot represent invalid state),
GT = generated test (structural but test-time, for registry/global invariants).

---

## 8. Execution Strategy

This is not a production repo. No external consumers, no backward
compatibility obligations.

1. **Behavior census** — classify every existing behavior into candidate
   roles. Frequency table + outliers. Output candidate roles + synonym
   clusters (check/verify/detect). Determines the initial pattern set
   and validates P3.

2. **Define core types** — `ContractNode`, `Patterns`, `PatternUse<T>`,
   `PropertyClaim`, `Verification` (including `Derived`), node/edge kinds,
   typed semantic enums, `Pattern` trait, `SubDag`, `PortGuard`,
   Lane B registry macros, `PatternInstanceId`.

3. **Delete old contracts internals** — remove current `Behavior`, `Property`,
   `UnderstandingDependency`, `Requirement`. Replace with new types.
   Everything breaks; everything gets rewritten.

4. **Rewrite tool understandings** — compose pattern specs, not annotate flat
   behaviors.

5. **Update generators** — consume new types directly. No facade.

The blast radius (18 crates, 37 files) is the work list, not a risk to
manage.

---

## 9. Open Questions

### How many patterns ship in V2?

The behavior census (step 1) determines this. Currently one pattern (upsert)
is formalized. Lifecycle may collapse into upsert. Until census data shows
otherwise, assume one pattern and build the machinery to support N.

### Pattern discovery process

When a new tool doesn't fit existing patterns, what's the process?

1. Declare the pattern in Lane B via `declare_pattern!()` — typed,
   structural, auditable, but not in the core `Patterns` struct.
2. Implement the first tool against the Lane B pattern.
3. When a second tool needs the same pattern, promote to Lane A (add a
   `PatternUse<T>` field to `Patterns`).

This means the first tool is never blocked, but the core vocabulary never
bloats without evidence.

### Guard semantics during lowering — resolved

The guard/skip mechanism (§3) lowers to **guarded blocks**: the `Block`
struct in GraphIR gains a `skip_predicate` field. The runner evaluates the
predicate before execution. If true, the block is marked `Skipped` and skip
propagates to downstream data-flow dependents.

This keeps the DAG topology static (simple for `gunbai-dag`) while making
execution dynamic. Alternatives considered and rejected:

- **Synthetic gate nodes**: bloats the graph with no-op nodes.
- **Conditional edges**: requires `gunbai-dag` to understand logic,
  violating the "dumb executor" principle.

### Platform resolution during lowering

`UpsertCreate` may carry platform-specific strategies (Linux, macOS, etc.).
The lowering phase resolves platform at compile time — only the relevant
platform's strategy enters GraphIR. The runtime DAG contains no dead code
for other platforms.

---

## References

| Topic | Location |
|-------|----------|
| Problem analysis | [`bl1-retrospective.md`](./bl1-retrospective.md) |
| String channel audit | `bl1-retrospective.md` §4 |
| Blast radius | `bl1-retrospective.md` §5 |
| Worked examples | [`v2-worked-examples.md`](./v2-worked-examples.md) |
| gunbai-dag | `crates/gunbai-dag/` |
| Go-era prior art | [`dag-systems-overview.md`](./dag-systems-overview.md) |
| Abstraction Calculus | Internal document — domain-agnostic calculus for towers of abstraction (inspiration for the lens/kernel-quotient structure underlying pattern instantiation and the three-level DAG as re-priming steps) |
