# Design — Lens framework (lens-as-parametric-monoid)

**Status:** `PROPOSAL` (skeleton authored 2026-04-28 by PM with Director-locked design decisions; Director-authored post-#1078-merge for full spec).

**Authority on promotion:** parent design-emission-model.md §"Modeling problem 8 — cost lens over emission" + §"Open call 4 — Lens-as-parametric-monoid framework"; this doc is the separate authority for the *general* framework, not just the cost-lens instance.

**Why a separate doc:** `docs/design-emission-model.md` is about emission specifically. The lens-as-parametric-monoid framework is broader — covers complexity, coercion-cost, capability-flow, IFC, etc. Per [Director response on PR #1078 (2026-04-28)](https://github.com/gunb-ai/gunbc/pull/1078), the framework is its own design authority to keep single-authority discipline clean.

## Frame

**Load-bearing claim:** every lens that folds over the compositional DAG is a *catamorphism* parametrized by a (cost-basis, monoid, side-condition) tuple. The compiler provides one fold; lens authors provide the algebra. **Each new lens is O(1) work**: declare the cost basis, declare the monoid, declare the side-condition; the fold is free.

This is the structural primitive that THESIS implies but doesn't make explicit:
- **THESIS §"User-defined dimensions"** — user lenses use the same mechanism as built-ins → that mechanism is the parametric fold
- **THESIS §"Concept unification"** — coercion cost = complexity → both are instances of the same lens framework with different cost bases
- **THESIS §"Free consequences"** — lenses fall out from substrate closure → the framework is what makes them fall out

**Why "free for coercion" generalizes:** the cost-lens-over-emission unification ("coercion cost = complexity") is one instance of the framework. **Every lens should be free for its domain.** If a lens needs custom fold machinery, that's a structural modeling gap — fix the substrate, not the lens.

## The Lens<C> primitive

```
Lens<C> = {
  name:            String                          // dimension identifier (populates DimensionReport.dimension_name)
  read:            (Dag, Behavior) → Witness<C>   // per-Behavior cost basis; Dag for substrate-fact lookup; typed failure channel
  sequential:      Monoid<C>                       // sequential composition (BindNode); structural inhabitance of Monoid<C> from dsl/std/algebra.dag:110 — projects to op = compose, identity = unit; monoid law (associativity + identity) is structurally enforceable, not just documented
  branch:          (C, C) → C                      // exclusive choice (BranchNode); only one arm runs — cost composition is max/join over arms; NOT a monoid op (no identity element required since branch always combines two arms)
  iterate:         (C, LoopBound) → C              // bounded iteration (LoopNode); LoopBound from src/v3/std/substrate.dag:316
  validate:        (Dag, C) → OptionalDiagnostic   // aggregate side-condition; Dag for workflow/sink lookup; aggregate location via Diagnostic.span
}
```

`Dag` and `Behavior` are the existing substrate types from `src/v3/std/substrate.dag` (no new substrate type introduced). `read`'s `(Dag, Behavior)` signature matches the existing `AnalysisDimension.witness_of: fn(Dag, Behavior) -> Witness<Carrier>` at `dimensions.dag:84` verbatim — no hidden global lookup authority for substrate facts; the `Dag` parameter carries the lookup context explicitly.

**Sequential composition uses `Monoid<C>` structurally** (per Director directive 2026-04-28 + gpt-5-5-pro reflective analysis Finding #4): the prior parallel `compose: (C, C) → C` + `unit: C` field pair was duplicating `Monoid<C>` (`dsl/std/algebra.dag:110-113` — `op + identity`) — same drift shape as `AnalysisDimension<Carrier>` previously carried. Replacing parallel fields with a single `sequential: Monoid<C>` field via **structural inhabitance** (per `feedback_epistemic_stacking` — every concept attaches to ontological DAG; same modeling-discipline move as Q1's `Interval<D>` consolidation) means: (i) the monoid law (associativity + identity) is *structurally* enforceable instead of merely documented; (ii) downstream consumers project from the monoid_witness rather than reading two parallel fields; (iii) future algebraic refinements (CommutativeMonoid for unordered sequential composition; Group for invertible composition) attach by extending the parent without changing Lens<C>'s shape. **`AnalysisDimension<Carrier>` followed the same collapse on 2026-05-04 (F2 / PR #1607);** `dimensions.dag` now carries `compose: Monoid<Carrier>` directly.

`branch` is NOT a monoid op — exclusive choice doesn't require an identity element (you always combine two arms; there's no "no-op branch" identity that would project to either arm). It's a standalone binary operation with `max/join` semantics. If a user-authored lens needed branch to have an identity (e.g., empty branch arm = unit), they could declare `branch: Monoid<C>` for their instance — but the framework doesn't require it.

**Substrate naming note:** the analysis-dimension framework is named `AnalysisDimension<Carrier>` in the substrate (`src/v3/std/dimensions.dag:82`). DB-3 design doc ([`docs/design-dimension-abstraction.md`](design-dimension-abstraction.md)) refers to the same concept colloquially as `Dimension<Carrier>`; the substrate disambiguated by prefixing `Analysis-` to avoid collision with `Dimension<Unit, Carrier>` (the typed-value-wrapper for `Duration<Seconds>`-shaped values, `dimensions.dag:98`). When this lens-framework doc cites a substrate type, the substrate name is authoritative; design-doc colloquial names are not.

**Why three composition operations and not four** (per codex BLOCKING finding on `71db19db`): v3 has three composition primitives among the 5 L1 behaviors — `BindNode` (sequential), `BranchNode` (exclusive choice), `LoopNode` (bounded iteration). There is no `ParallelNode`. Auto-parallelism per THESIS §"Free consequences" is **emergent** from dependency-graph analysis on `BindNode` sequences (independent binds run in parallel; data-dependent binds are serialized), not a declared substrate primitive. The lens framework declares operations over the actual L1 behaviors, so `branch` (exclusive choice — `max/join`) appears, not a phantom `parallel` (which would mis-encode L1 semantics under P2/P6). Lens authors who want a parallelism cost facet declare it as a *derived* property of `compose` over independent sub-DAGs, or as a separate lens instance — not via a new substrate field.

**Why `read` returns `Witness<C>` and not `C`** (per codex BLOCKING finding on `4057b8f5`): a `(Dag, Behavior) → C` signature has no typed failure channel — a missing per-Behavior substrate fact (e.g., a primitive without a declared cost) would have to either fabricate a default `C` (violates fail-closed) or panic (violates fail-closed). `Witness<C>` from `src/v3/std/dimensions.dag:35-37` is the existing typed channel:

```
type Witness<Carrier>
  = Inhabits(Carrier)                                   // fact present; carries the read value
  | Violates { reason: String, at: Behavior }           // fact missing; structured failure with per-Behavior reference
```

The fold accumulates `Witness<C>` outputs from `read`. Any `Violates` becomes a `Diagnostic` in the final `DimensionFail.violations` — no fabricated carrier ever reaches `compose`.

**Why `validate` takes `(Dag, C)` and returns `OptionalDiagnostic`** (per codex BLOCKING findings on `c9898163` + `5d318eca1`): aggregate validation needs (1) the aggregate `C` (post-compose) and (2) workflow/sink declarations from the program structure. The `Dag` parameter is exactly that program-structure context — workflow capability grants, sink clearance declarations, and other side-condition facts are all `.dag` declarations the validator looks up structurally. Without `Dag`, validate would need hidden context or fabricated locations.

`Witness<C>.Violates { reason, at: Behavior }` carries a per-Behavior reference — appropriate for `read`'s per-Behavior lookups, but **wrong for aggregate validation**. By the time `validate` runs, the fold has composed many Behaviors' Witnesses into a single aggregate `C`; there is no single Behavior to put in `at`. Reusing `Witness<C>` here would force fabricating `at: Behavior` (parallel-representation debt + fail-closed violation) or losing location info entirely. The right shape is `OptionalDiagnostic` from `src/v3/std/dimensions.dag:41-43`:

```
type OptionalDiagnostic
  = NoDiagnostic                                        // validation passed
  | SomeDiagnostic { value: Diagnostic }                // validation failed; aggregate-level diagnostic
```

`Diagnostic` carries `span: SourceSpan` — the natural location for an aggregate validation failure (the workflow root binding, the sink declaration, etc.), not a per-node Behavior. This matches the existing `AnalysisDimension.break_diagnostic: fn(Behavior, Carrier) -> OptionalDiagnostic` pattern at `dimensions.dag:86` (note: `break_diagnostic` takes a per-Behavior input but the lens-framework `validate` is purely aggregate-level, so it drops the Behavior parameter — only `OptionalDiagnostic` shape is reused).

The fold lifts a `SomeDiagnostic { value }` into `DimensionFail.violations`; `NoDiagnostic` lets the fold produce `DimensionOk`.

A lens is a *generic algebra* over the 5 L1 behaviors. The compiler provides:

```
fold_lens<C>: Lens<C> → Dag → DimensionReport<C>
```

where `DimensionReport<Carrier>` is the **existing** carrier already declared at `src/v3/std/dimensions.dag:51-61`. Verbatim from that file:

```
type DimensionReport<Carrier>
  = DimensionOk {
      dimension_name: String
      composed: Carrier
      witnesses: List<Witness<Carrier>>
    }
  | DimensionFail {
      dimension_name: String
      violations: List<Diagnostic>
      witnesses: List<Witness<Carrier>>
    }
```

The lens framework **reuses this carrier verbatim** — no parallel `LensReport` shape, no rename of variants, no rename of the `violations` field. Lens authors get the same fail-closed partition (`DimensionOk` carries `composed`; `DimensionFail` carries `violations: List<Diagnostic>` and never fabricates a carrier). `Witness<Carrier>` is also reused from `dimensions.dag:35-37` (`Inhabits(Carrier) | Violates { reason, at }`).

**Carrier reconciliation note (2026-04-28 per codex BLOCKING finding on `4057b8f5`):** an earlier draft of this doc projected a desired `Satisfied { composed, witnesses } | Violated { diagnostics: List<EmissionDiagnostic> }` shape that did not match what's actually declared in `dimensions.dag`. That was parallel-representation debt — the lens framework would either need a separate type (rejected, parallel-representation) or `dimensions.dag` would need a rename cascade (out of scope for the lens primitive lane). **Resolution:** the framework consumes the existing `DimensionReport<Carrier>` as-is. If lens-specific failure data (e.g., richer than `Diagnostic`) is ever needed, that's a follow-up modeling question for `dimensions.dag` itself, not a lens-framework concern.

No silent fabrication; typed diagnostics on side-condition failure (via `Diagnostic` variants on `DimensionFail.violations`).

## Director-locked design decisions (2026-04-28)

Per Director response on #1078, the following are LOCKED for the framework spec:

1. **Pure monoidal**, not stateful. Memory-peak (anamorphism + state-passing) is a *separate* framework — don't conflate. Stateful folds get their own primitive when needed (post-R3).
2. **Result type:** the existing `DimensionReport<Carrier> = DimensionOk { dimension_name, composed, witnesses } | DimensionFail { dimension_name, violations: List<Diagnostic>, witnesses }` from `src/v3/std/dimensions.dag` is reused verbatim — no parallel `LensReport` type, no variant rename. Lens-framework spec consumes the existing carrier as authoritative. Failure carries `violations: List<Diagnostic>` (not a separate `EmissionDiagnostic` type). No silent fabrication.
3. **Higher-order shapes:** function-valued cost basis derived from signature. **Meta-lens (lens-on-lens) deferred post-R3** — solves a problem we don't have at structural close.
4. **Cross-domain composition:** explicit declaration only. `Lens<C> × Lens<D> = Lens<(C, D)>` with product monoid; side-conditions compose conjunctively. **User-declared, not auto-derived.**
5. **User-authored lens substrate:** T-LensAPI rescope to lens-as-monoid in same wave as `Lens<C>` lands. User-lens surface inherits structurally — same primitive for built-in and user-authored.
6. **Three worked instances sufficient for generality validation:** complexity (additive numeric monoid) + tenant-flow (set union + categorical authorization) + IFC (lattice join + downgrade rejection). Stretch goals (memory-peak, energy, latency) are post-R3 instances; the three cover the structural axes.

## Three worked instances

Each instance demonstrates a different monoid shape and validates the framework's range. All three share the same fold machinery; they differ only in (`read`, `sequential` (the Monoid<C> witness), `branch`, `iterate`, `validate`). Worked traces below colloquially say "compose" when describing the monoid op; the substrate field is `sequential: Monoid<C>` and `compose` is `sequential.op`, `unit` is `sequential.identity`.

### Instance 1 — Complexity (additive numeric monoid)

Establishes the basic framework shape.

**Cost basis declaration:**
```
type SymbolicCost = CostExpr {
  work:              SymbolicExpr   // total work (sum of operations)
  span:              SymbolicExpr   // critical-path length (max across exclusive Branch arms; max-then-add across Bind sequence)
  asymptotic_class:  BigOClass      // O(1), O(log n), O(n), O(n log n), O(n^2), ...
}
```

**SymbolicCost algebra witnesses** (per Director directive 2026-04-28 + gpt-5-5-pro reflective analysis Finding #6 — declare the algebra explicitly so consumers can compose generically; `feedback_modeling_philosophy` discipline):

```
// Sequential composition: SymbolicCost forms a Monoid under work-additive,
// span-additive, asymptotic-max composition. Identity is the zero-cost cost.
inhabits SymbolicCost : Monoid<SymbolicCost>
  op(a, b) = CostExpr(work=a.work + b.work, span=a.span + b.span,
                       class=max(a.class, b.class))
  identity = CostExpr(work=0, span=0, class=O(1))

// Branch composition (NOT a monoid op): SymbolicCost forms a JoinSemilattice
// under work-max, span-max, asymptotic-max — exclusive choice takes worst
// case across arms. Branch isn't required to have an identity; if a user
// instance needs one, they can declare BoundedJoinSemilattice (with bottom).
inhabits SymbolicCost : JoinSemilattice<SymbolicCost>
  join(a, b) = CostExpr(work=max(a.work, b.work), span=max(a.span, b.span),
                         class=max(a.class, b.class))

// Asymptotic class lattice: BigOClass is itself a BoundedLattice with
// O(1) at the bottom and an unbounded top. Used by branch.join above.
inhabits BigOClass : BoundedLattice<BigOClass>
  meet = min  identity_meet = O(unbounded)
  join = max  identity_join = O(1)
```

The lens framework reads these inhabitance witnesses via `Dag::declarations()` — the `compose` action in the Lens<SymbolicCost> instance below projects from `Monoid<SymbolicCost>.op`, not from a free-standing function.

**Lens<SymbolicCost> declaration:**

| Field | Definition |
|---|---|
| `name` | `"complexity"` |
| `read(dag, behavior)` | Reads operation's declared cost from `dsl/std/algebra.dag` (looked up via `dag` context for the given `behavior`) and wraps as `Inhabits(...)` (e.g., `OrderedRing.add` → `Inhabits(CostExpr(1, 1, O(1)))`); returns `Violates { reason: "no declared cost for <op>", at: behavior }` if the substrate has no fact for that operation |
| `sequential` (Bind; `Monoid<SymbolicCost>`) | `op(a, b) = CostExpr(work=a.work + b.work, span=a.span + b.span, class=max(a.class, b.class))`; `identity = CostExpr(work=0, span=0, asymptotic_class=O(1))`. Monoid law: associativity holds (work/span/class composition is associative); identity holds (composing with zero-cost is identity). |
| `branch` (BranchNode) | `CostExpr(work=max(a.work, b.work), span=max(a.span, b.span), class=max(a.class, b.class))` — exclusive choice = worst case across arms (only one arm runs at runtime; the lens conservatively reports the worst); NOT work-additive |
| `iterate(body, loop_bound)` (LoopNode; `loop_bound: LoopBound`) | `CostExpr(work=body.work × loop_bound, span=body.span × loop_bound, class=multiply_class(body.class, loop_bound))` |
| `validate(dag, c)` | Always `NoDiagnostic` — complexity has no side-condition; `dag` is unused; final result is `DimensionOk` if all reads `Inhabits` |

**Worked program:**
```
data x = compute_pair(in)             // bind: O(n)
data y = branch { case_a: sort(x), case_b: aggregate(x) }   // exclusive choice; only one arm runs
data z = compute_summary(y, in)        // bind: O(1)
```

**Fold trace** (every `read` returns `Witness<CostExpr>`; the fold unwraps `Inhabits(c)` and accumulates witnesses; any `Violates` would short-circuit to `DimensionFail`):
1. `read(compute_pair)` → `Inhabits(CostExpr(n, n, O(n)))`
2. `read(sort)` → `Inhabits(CostExpr(n log n, log n, O(n log n)))`
3. `read(aggregate)` → `Inhabits(CostExpr(n, n, O(n)))`
4. `branch(sort, aggregate)` → `CostExpr(work=max(n log n, n)=n log n, span=max(log n, n)=n, class=max(O(n log n), O(n))=O(n log n))` — exclusive choice takes worst case across arms (compose pulls inhabited values; same for steps 5/7)
5. `compose(compute_pair, branch(...))` → `CostExpr(work=n + n log n, span=n + n, class=O(n log n))` — sequential composition adds work and adds span
6. `read(compute_summary)` → `Inhabits(CostExpr(1, 1, O(1)))`
7. `compose(...)` → final = `CostExpr(O(n log n) work, O(n) span, O(n log n) class)`

**Result:** `DimensionOk { dimension_name: "complexity", composed: CostExpr(work=O(n log n), span=O(n), class=O(n log n)), witnesses: [...per-step...] }`

**Coercion-cost-as-instance:** the coercion case (Modeling problem 8 in design-emission-model.md) is the same `Lens<SymbolicCost>` with a *different* `read` function — instead of reading from `algebra.dag`, it reads from per-target language spec (Rust `u32.add` → CostExpr(1, 1, O(1)); Rust `BigInt.add` → CostExpr(digits, digits, O(n))). **Same fold; different cost-basis source.** That's the unification falling out structurally.

### Instance 2 — Tenant-flow (set union + categorical authorization)

Demonstrates non-numeric monoid + side-condition.

**Cost basis declaration:**
```
type CapSet = Set<Capability>
type Capability = Read(TenantId) | Write(TenantId) | Network | Filesystem | ...
```

**Lens<CapSet> declaration:**

| Field | Definition |
|---|---|
| `name` | `"tenant-flow"` |
| `read(dag, behavior)` | Reads operation's declared capability requirement set (looked up via `dag` for the given `behavior`) wrapped as `Inhabits(...)` (e.g., `read[TenantA].orders` → `Inhabits({Read(TenantA)})`); returns `Violates { reason: "no declared capability requirement for <op>", at: behavior }` if the substrate has no fact for that operation |
| `sequential` (Bind; `Monoid<CapSet>`) | `op = set union`; `identity = {}` (empty set). Monoid law: union is associative + commutative (so this is actually `CommutativeMonoid<CapSet>` — Lens<C> only requires monoid; the commutative refinement is available structurally if a consumer needs it). |
| `branch` (BranchNode) | Set union — exclusive choice; only one arm runs but compile-time analysis doesn't know which, so defensive accumulation: program must be granted capabilities for any arm it might take |
| `iterate(body, loop_bound)` (LoopNode; `loop_bound: LoopBound`) | Body's CapSet (the loop bound doesn't matter for capability set — every iteration requires the same caps as the body) |
| `validate(dag, set)` | Reads `workflow.cap_grant` from `dag` (the `@cap_grant(...)` declaration on the workflow root). If `set ⊆ workflow.cap_grant`: return `NoDiagnostic`. Otherwise: return `SomeDiagnostic { value: Diagnostic { kind: CapabilityViolation, span: <workflow root span — read from dag>, message: "required: <set>, granted: <workflow.cap_grant>, missing: <set ∖ granted>", ... } }` (the `CapabilityViolation` kind is a lens-local diagnostic-kind declaration landed alongside this lens instance; `span` and grant data come from `dag`, not hidden context) |

**Worked program:**
```
@cap_grant({Read(TenantA), Write(TenantA)})
data orders = read[TenantA].orders     // CapSet: {Read(TenantA)}
data report = aggregate(orders)         // CapSet: {} (pure)
data summary = write[TenantB].report    // CapSet: {Write(TenantB)}
```

**Fold trace** (every `read` returns `Witness<CapSet>`; the fold unwraps `Inhabits(c)` to feed compose; any `Violates` would short-circuit to `DimensionFail`):
1. `read(read[TenantA].orders)` → `Inhabits({Read(TenantA)})`
2. `read(aggregate)` → `Inhabits({})`
3. `read(write[TenantB].report)` → `Inhabits({Write(TenantB)})`
4. `compose(...)` → `{Read(TenantA), Write(TenantB)}`
5. `validate({Read(TenantA), Write(TenantB)})` against grant `{Read(TenantA), Write(TenantA)}`:
   - Missing: `{Write(TenantB)}` — returns `SomeDiagnostic { value: Diagnostic { kind: CapabilityViolation, span: <workflow root>, message: "required: {Read(TenantA), Write(TenantB)}, granted: {Read(TenantA), Write(TenantA)}, missing: {Write(TenantB)}", ... } }`
6. **Result:** `DimensionFail { dimension_name: "tenant-flow", violations: [<the Diagnostic from step 5>], witnesses: [...] }` (fold lifts `SomeDiagnostic.value` into `violations`)

The lens fail-closes structurally: the program crosses a tenant boundary that requires explicit grant. **Same fold framework as complexity; different monoid (set union); side-condition enforces the categorical authorization check.**

### Instance 3 — Information Flow Control (lattice join + downgrade rejection)

Demonstrates lattice-typed monoid + categorical side-condition (different from set + side-condition).

**Cost basis declaration:**
```
type SecurityLabel = Public | Confidential | Secret | TopSecret
// Lattice: Public ⊏ Confidential ⊏ Secret ⊏ TopSecret
// (declared in dsl/std/security.dag as BoundedLattice<SecurityLabel>)
```

**Lens<SecurityLabel> declaration:**

| Field | Definition |
|---|---|
| `name` | `"ifc"` |
| `read(dag, behavior)` | Reads operation's data label (looked up via `dag` for the given `behavior`) wrapped as `Inhabits(...)` (e.g., `read[TopSecret].records` → `Inhabits(TopSecret)`); returns `Violates { reason: "no declared security label for <op>", at: behavior }` if the substrate has no fact for that operation |
| `sequential` (Bind; `Monoid<SecurityLabel>`) | `op = lattice join (max)`; `identity = Public` (lattice bottom — the BoundedLattice<SecurityLabel> bottom inhabitance). Monoid law: lattice join is associative + commutative + idempotent (this is `BoundedSemilattice` actually — refinement of Monoid via `BoundedLattice<SecurityLabel>` from `dsl/std/algebra.dag:222`). |
| `branch` (BranchNode) | Lattice join (`max`) — exclusive choice; defensive worst-case label across arms (only one arm runs but compile-time analysis must allow for either) |
| `iterate(body, loop_bound)` (LoopNode; `loop_bound: LoopBound`) | Body's label (the loop bound doesn't matter for IFC labels — every iteration produces data with the same label as the body) |
| `validate(dag, label)` | Reads sink declaration + clearance (`@sink_clearance(...)`) from `dag`. If `label ⊑ sink.label`: return `NoDiagnostic`. Otherwise: return `SomeDiagnostic { value: Diagnostic { kind: IFCDowngradeViolation, span: <sink declaration span — read from dag>, message: "computed: <label>, sink_clearance: <sink.label>, downgrade_required: <label ⊐ sink.label>", ... } }` (the `IFCDowngradeViolation` kind is a lens-local diagnostic-kind declaration landed alongside this lens instance; `span` and clearance data come from `dag`, not hidden context) |

**Worked program:**
```
@sink_clearance(Confidential)
data secret_data = read[TopSecret].records     // label: TopSecret
data report = aggregate(secret_data)             // label: TopSecret (join with Public unit)
data output = write[Sink].report                 // label: TopSecret (sink expects Confidential)
```

**Fold trace** (every `read` returns `Witness<SecurityLabel>`; the fold unwraps `Inhabits(c)` to feed compose; any `Violates` would short-circuit to `DimensionFail`):
1. `read(read[TopSecret].records)` → `Inhabits(TopSecret)`
2. `read(aggregate)` → `Inhabits(Public)` (unit; pure operation has no data label of its own)
3. `compose(TopSecret, Public)` → `TopSecret` (lattice join = max)
4. `read(write[Sink].report)` → `Inhabits(Public)` (the sink itself doesn't carry data; its label is the clearance constraint, applied via `validate`)
5. `compose(TopSecret, Public)` → `TopSecret`
6. `validate(TopSecret)` against sink clearance `Confidential`:
   - `TopSecret ⊐ Confidential` → returns `SomeDiagnostic { value: Diagnostic { kind: IFCDowngradeViolation, span: <sink declaration>, message: "computed: TopSecret, sink_clearance: Confidential, downgrade_required: true", ... } }`
7. **Result:** `DimensionFail { dimension_name: "ifc", violations: [<the Diagnostic from step 6>], witnesses: [...] }` (fold lifts `SomeDiagnostic.value` into `violations`)

The lens enforces lattice-based information flow without explicit declassification. **Same fold framework as complexity + tenant-flow; different monoid (lattice join); side-condition enforces the lattice-ordered authorization check (different from set authorization).**

### Generality validation

The three instances cover three distinct monoid shapes:

| Instance | Monoid | Side-condition | Failure mode |
|---|---|---|---|
| Complexity | Additive numeric (work + span + class) | None (always `DimensionOk`) | n/a |
| Tenant-flow | Set union | Set difference against grant | Capabilities missing from grant |
| IFC | Lattice join (`max` on lattice order) | Lattice comparison against clearance | Downgrade required without grant |

If the framework supports all three, it supports any monoid + (optional) side-condition. **Stretch goals (memory-peak, latency, energy) are post-R3 instances** — they don't add new structural axes; they just instantiate the existing primitive at additional cost bases.

## Migration plan — 4 existing PROXY/STUB lenses → Lens<C> instances

The R2-T-Substrate-Lens-Primitive lane delivers `Lens<C>` and migrates the existing 4 PROXY/STUB lenses as instances. **Interface ON TOP of existing work**, not green-field — most of the patterns are already there in monoidal shape (`combine_max`, `combine_sequential` in `cost.dag`).

| Existing lens | Current state | Migration target | Sizing |
|---|---|---|---|
| `src/v3/lenses/cost.dag` | PROXY — has `combine_iterate` / `combine_max` / `combine_sequential` / `combine_dominant` parametrized only on `SymbolicCost` | `Lens<SymbolicCost>` instance — generic-ize the existing combinators | ~1-2 days (mostly generalization of existing patterns) |
| `src/v3/lenses/complexity.dag` | PROXY — single integer depth per port | `Lens<Depth>` instance with `Depth = Int`, `compose = max + 1` (sequential adds depth), `branch = max` (exclusive choice takes deeper arm) | ~1 day |
| `src/v3/lenses/idempotency.dag` | STUB — Rust oracle | `Lens<IdempotencyVerdict>` instance with `IdempotencyVerdict = IsIdempotent | IsBreaking(Reason)` and `compose = first-breaker-wins` | ~1-2 days |
| `src/v3/lenses/parallelism.dag` | STUB — fail-closed placeholder | `Lens<ParallelismVerdict>` instance — substrate-completion work | ~2-3 days |

**Total migration: ~5-8 days at gunbc velocity**, parallelizable across ~2 workers. Combined with the substrate primitive (~1-2 weeks), total R2-T-Substrate-Lens-Primitive lane sizing: **~1.5-2 weeks**.

## Up-front validation checklist (design + implementation phases)

Per user direction 2026-04-28: "i would also start going through the work we'll need to do up front to validate the design/implementation phase (basically so we can self check we're on the right path)."

This section enumerates the validation work that must happen *before* substrate worker dispatch begins. Each item is a self-check — if it fails, the design has a gap that needs reframing before implementation proceeds.

### Design-phase self-checks

These run as paper exercises (no code) before any `.dag` substrate work begins. Failure here means the design isn't ready.

**D1. Three worked examples each pass the fold by construction.**
- Walk through Instance 1 (complexity) on a 3-step program. Verify the fold output matches the expected `DimensionOk { dimension_name: "complexity", composed: ..., witnesses: ... }`.
- Walk through Instance 2 (tenant-flow) on a cross-tenant program. Verify the fold produces `DimensionFail` when expected.
- Walk through Instance 3 (IFC) on a TopSecret-to-Confidential leak. Verify lattice-comparison rejects as expected.
- **Pass criterion:** each fold trace matches expected output. Failure = monoid or side-condition spec is wrong.

**D2. Existing PROXY lenses fit `Lens<C>` shape (paper exercise).**
- For each of the 4 existing lenses, write the instance declaration (`name`, `read`, `sequential: Monoid<C>` — projects to `sequential.op` for sequential composition + `sequential.identity` for unit, `branch`, `iterate`, `validate`) using only existing combinators where possible.
- **Pass criterion:** every existing combinator (`combine_max`, `combine_sequential`, etc.) maps to a Lens<C> field with no machinery left over. Failure = the existing patterns aren't actually monoidal, or the framework needs additional fields.

**D3. L6 (structural-form coverage) is NOT a `Lens<C>` instance** (corrected per codex BLOCKING 2026-04-28T21:26).

- L6's fold ranges over `(substrate-form × Shape-A-target)` pairs, not per-Behavior. `Lens<C>.read: (Dag, Behavior) → Witness<C>` reads per-Behavior substrate facts; the input space doesn't match L6's per-(form × target) input space.
- L6 is a substrate-load-time completeness check (separate primitive) that walks every (connective × behavior × target) cell and verifies an emission-path declaration exists.
- L6 emits `Diagnostic` (kind = `MissingEmissionPath`) on each gap; substrate-load fails closed.
- **Pass criterion:** L6 lives as its own primitive in R2-T-Ground-CrossTarget-Meta scope. The lens framework does NOT cover L6; the framework covers per-Behavior structural folds only.

**D4. Cross-domain composition works for the 3 instances.**
- Hypothetical: a program has both complexity claims AND IFC claims.
- Compose `Lens<SymbolicCost> × Lens<SecurityLabel> = Lens<(SymbolicCost, SecurityLabel)>`.
- Verify side-conditions compose conjunctively (both must satisfy).
- **Pass criterion:** the composed lens correctly identifies a program that satisfies complexity but violates IFC (or vice versa). Failure = product monoid spec is wrong.

**D5. DimensionReport<C> covers all failure modes.**
- Enumerate failure modes across the 3 instances:
  - **Read-channel failures** (every instance): substrate has no fact for a node — `read` returns `Violates { reason, at }`; fold short-circuits to `DimensionFail` with that violation surfaced as a `Diagnostic` (kind = `MissingSubstrateFact` or per-instance variant)
  - **Validate-channel failures** (per instance): complexity has none; tenant has `CapabilityViolation`; IFC has `IFCDowngradeViolation`
- Verify each maps to a `Diagnostic` value with an appropriate lens-local diagnostic-kind declaration.
- **Pass criterion:** no failure mode requires fabricating a result; all surface as typed diagnostics — including the read-channel failure mode (per codex BLOCKING finding on `4057b8f5`: `read` must return `Witness<C>`, not `C`, so missing substrate facts cannot fabricate a `C` before validation).

**D6. Director's 6 locked decisions hold under examples.**
- Decision 1 (pure monoidal): verify none of the 3 examples requires state-passing. If memory-peak is needed, name it as separate framework.
- Decision 2 (DimensionReport result): verify all 3 examples produce one of the two variants.
- Decision 3 (no meta-lens): verify the examples don't apply lens-on-lens.
- Decision 4 (explicit cross-domain): verify D4 works without auto-derivation.
- Decision 5 (T-LensAPI rescope): verify a user-authored lens has the same shape as built-ins.
- Decision 6 (3 instances sufficient): verify covering complexity / tenant / IFC validates the abstraction.
- **Pass criterion:** all 6 hold under the worked examples. Failure = revisit the locked decision.

### Implementation-phase self-checks

These run during substrate-worker dispatch, before declaring the lane closed. Each is a TestClaim-shaped acceptance.

**I1. `dsl/std/lens.dag` declares `Lens<C>` and type-checks against existing substrate.**
- Lens<C> declaration includes the 6 fields (`name`, `read`, `sequential: Monoid<C>`, `branch`, `iterate`, `validate`). `sequential` carries the monoid witness via structural inhabitance; `unit` and `compose` project from `sequential.identity` and `sequential.op` rather than appearing as separate Lens fields (per Director synthesis 2026-04-28: parallel `compose + unit` fields would duplicate `Monoid<C>` and reintroduce the AnalysisDimension drift).
- `read: (Dag, Behavior) → Witness<C>` (typed per-Behavior failure channel; matches `AnalysisDimension.witness_of: fn(Dag, Behavior) -> Witness<Carrier>` at `dimensions.dag:84` verbatim).
- `validate: (Dag, C) → OptionalDiagnostic` (aggregate-level failure channel; `Dag` for workflow/sink declaration lookup; `OptionalDiagnostic` from `dimensions.dag:41-43`; location info via `Diagnostic.span: SourceSpan`, not per-Behavior).
- All `read` and `validate` lookups go through the explicit `Dag` parameter — no hidden global lookup authority.
- Type-checks against existing `BoundedLattice<T>`, `DimensionReport<Carrier>`, `Witness<Carrier>`, `OptionalDiagnostic`, `Dag`, `Behavior`, and `LoopBound` types from `src/v3/std/`.
- **Pass criterion:** substrate parses; structural-form ratchet remains green; no fabricated-carrier path in `read`; no fabricated-`Behavior` path in `validate`; no hidden global lookups (every substrate fact accessed through `Dag` parameter).

**I2. Generic fold machinery `fold_lens<C>` is small.**
- Implementation in `src/v3/std/lens.dag` (or equivalent) is ≤ 200 lines.
- Reads only Lens<C> + Dag + DimensionReport<C> primitives; no per-instance branching.
- **Pass criterion:** code review confirms no instance-specific logic in the fold. Failure = the fold isn't actually generic.

**I3. Migration of 4 existing PROXY lenses mechanical.**
- Each migration is a single PR ≤ 100 lines diff (instance declaration + delete redundant combinators).
- **Pass criterion:** all 4 migrations land cleanly; no consumer refactoring required.

**I4. Three worked-example fixtures pass.**
- Authored as `.dag` `TestClaim` declarations:
  - `lens_complexity_n_log_n_fold_correct`: walks Instance 1's worked program; expects CostExpr(O(n log n), O(n), O(n log n))
  - `lens_tenant_flow_cross_tenant_violated`: walks Instance 2's worked program; expects CapabilityViolation diagnostic
  - `lens_ifc_topsecret_to_confidential_violated`: walks Instance 3's worked program; expects IFCDowngradeViolation diagnostic
- **Pass criterion:** all 3 TestClaims evaluate true.

**I5. Cross-domain product fixture passes.**
- TestClaim `lens_product_complexity_x_ifc_correct`: program that satisfies complexity but violates IFC, composed via `Lens<C> × Lens<D>`, returns `DimensionFail` with the IFC `Diagnostic` in `violations`.
- **Pass criterion:** product fold + conjunctive side-condition behaves correctly.

**I6. L6 lives as its own substrate-load-time completeness check** (corrected per codex BLOCKING 2026-04-28T21:26 — L6 is NOT a Lens<C> instance because Lens<C>.read input space is per-Behavior; L6's input space is per-(form × target)).
- TestClaim `l6_completeness_check_walks_all_cells`: substrate-load-time completeness check walks every (connective × behavior × target) cell and verifies an emission-path declaration exists; emits typed Diagnostic on gaps.
- **Pass criterion:** L6 acceptance gate `l6_structural_form_coverage` passes via the cross-product completeness check primitive in R2-T-Ground-CrossTarget-Meta scope. NOT via the lens framework.

**I7. User-authored lens TestClaim.**
- A user (test author) declares a custom `Lens<MyCostBasis>` with their own monoid; runs the fold; gets a result.
- TestClaim `user_authored_lens_via_framework`: confirms the user-lens surface uses the same machinery as built-ins.
- **Pass criterion:** no special path for user lenses vs built-ins.

**I8. Read-channel fail-closed TestClaim** (added per codex BLOCKING finding on `4057b8f5`).
- Construct a program that uses an operation for which the substrate has no declared cost/capability/label fact (whichever instance is being tested).
- Run the lens fold; expect `DimensionFail` with `violations` containing a `Diagnostic` whose kind names the missing fact. The Witness's `at: Behavior` carries per-node location for read-channel failures.
- TestClaims:
  - `lens_complexity_missing_cost_fail_closed`: fold on a program with an op lacking declared cost → `DimensionFail`
  - `lens_tenant_flow_missing_cap_fail_closed`: fold on a program with an op lacking declared capability requirement → `DimensionFail`
  - `lens_ifc_missing_label_fail_closed`: fold on a program with an op lacking declared security label → `DimensionFail`
- **Pass criterion:** every instance's `read` returns `Witness<C>`; missing facts produce typed diagnostics; no fabricated `C` ever reaches `compose`.

**I9. Aggregate-validate fail-closed TestClaim** (added per codex BLOCKING finding on `c9898163`).
- Construct programs that pass `read` (all per-node facts present) but fail aggregate validation (tenant grant missing a capability; sink clearance below computed label).
- Run the lens fold; expect `DimensionFail` with `violations` containing a `Diagnostic` whose `span: SourceSpan` points at the workflow root or sink declaration (aggregate-level location), not at any per-node `Behavior`.
- TestClaims:
  - `lens_tenant_flow_aggregate_validate_fail_closed`: validate detects missing capability; diagnostic span at workflow root
  - `lens_ifc_aggregate_validate_fail_closed`: validate detects clearance violation; diagnostic span at sink declaration
- **Pass criterion:** `validate: (Dag, C) → OptionalDiagnostic` returns `SomeDiagnostic { value: Diagnostic }` on failure; the fold lifts that into `DimensionFail.violations`; no `at: Behavior` is fabricated for aggregate-level failures; workflow/sink declarations and span values are read from the explicit `Dag` parameter, not from hidden context.

### Migration-phase self-checks

These ratchet the migration of existing lenses. Each existing lens has a target-state TestClaim.

**M1. `cost.dag` migration:** TestClaim `cost_lens_via_framework_correct`: post-migration `Lens<SymbolicCost>` instance produces the same fold output as pre-migration `combine_*` calls on a benchmark program.

**M2. `complexity.dag` migration:** TestClaim `complexity_lens_via_framework_correct`: similar, for the depth-per-port lens.

**M3. `idempotency.dag` migration:** TestClaim `idempotency_lens_via_framework_correct`: post-migration retires the Rust oracle; the `Lens<IdempotencyVerdict>` instance produces the same answer.

**M4. `parallelism.dag` migration:** TestClaim `parallelism_lens_via_framework_correct`: post-migration retires the fail-closed placeholder; the framework instance handles it.

**Cumulative pass:** all 4 migrations land + their TestClaims pass + the existing combinators (`combine_max`, etc.) are retired + SG-0 hand-Rust count drops by the lens-related Rust files.

## Open design questions surfaced by validation

Things to think through during design phase that could surface gaps:

1. **Witness construction for non-trivial monoids.** Complexity's witnesses are concrete CostExpr values; tenant-flow's witnesses are set-difference results; IFC's witnesses are lattice-comparison failures. Is `Witness<C>` general enough to encode all three? (Pre-D5 check.)

2. **Error recovery for partial failure.** If a program partially violates IFC (some paths leak, others don't), does the fold report all violations or stop at the first? Director's "no silent fabrication" rule says report all. Is that the spec?

3. **Lens-application performance.** The fold visits every Node in the DAG. For large programs, can we memoize? If so, on what key (Node identity? structural hash?). ~~This is post-R3 optimization; flagging here so we don't accidentally bake in non-memoizable shape.~~ **Director-ratified 2026-04-30 invariant (locks memoizable shape; defers implementation; corrected per #1319 BLOCKING review):** **`Lens.read` MUST be a pure function of its declared `(Dag, Behavior)` inputs** — that is, the function may freely consume substrate facts reachable through the `Dag` (per-operation cost from `dsl/std/algebra.dag`, capability-requirement sets, security labels, etc., as the worked instances at lines 155 / 196 / 237 / 343 demonstrate), but **MUST NOT depend on any external mutable state** (no global registries, no time, no I/O, no consumer-side caches). The `Dag` IS the substrate authority L-7/P2 require; the purity invariant is about *external* state, not about the substrate-fact-lookup channel. This locks memoizable shape (memoization key is `(Dag-identity, Behavior-identity)` since both inputs are immutable); runtime memoization of the lens fold is then an instance of the auto-memoization free consequence (T-Free-Consequences-Demonstration in R3). Same lane that demonstrates auto-parallelism + auto-memoization + cross-target opt + space-bound CX as structural consequences of the `Lens<C>` fold framework over the 5 substrate behaviors. Memoization implementation is the consequence demonstration; the *external-state purity invariant on `Lens.read`* is the substrate discipline that makes it possible.

4. **Parametric algebra interaction.** `Dimension<Carrier>` (R2 Modeling) is a parametric type. Does a `Lens<Dimension<...>>` make sense? Should be expressible if `Lens<C>` is fully parametric.

5. **Side-condition composition when one lens has none.** Complexity has no side-condition; IFC has one. `Lens<SymbolicCost> × Lens<SecurityLabel>` should still validate IFC correctly. Verify D4 covers this.

These are research-questions for the design phase; not blockers for the substrate primitive itself but worth recording.

## Design questions to lock before substrate dispatch

**Status:** SURFACED 2026-04-28 per Director directive — modeling per-language is hard; verification (testing) discipline must be built in; open-ended modeling fails (alias/clone class).

The questions below are pre-dispatch decisions for the lens framework. Each names alternatives, cascade implications, TestClaim shape, and recommendation. Director signoff before T-Substrate-Lens-Primitive dispatch.

### Q6 — `Witness<C>` generality across non-trivial monoids

**Status:** REFERENCED in §"Open design questions" item 1. Tenant-flow witnesses are set-difference results; IFC witnesses are lattice-comparison failures. Existing `Witness<C>.Violates { reason: String, at: Behavior }` carries a string `reason` and a `Behavior` reference.

**Question:** does the existing `Witness<C>` from `src/v3/std/dimensions.dag:35-37` generalize to non-trivial monoids (set-difference for tenant-flow; lattice-comparison for IFC), or does it need extension?

**Alternatives:**
- (a) Witness<C> is sufficient as-is. Set-difference/lattice-comparison results encode into `reason: String` (e.g., "missing capabilities: {Read(TenantA)}" or "TopSecret ⊐ Confidential"). String is opaque but lossy — downstream consumers can't programmatically extract the structural failure.
- (b) Extend `Witness<C>` to carry a typed failure payload: `Violates { reason: ViolationReason<C>, at: Behavior }` where `ViolationReason<C>` is an instance-specific sum type. Each lens instance declares its own ViolationReason variants.
- (c) Keep `Witness<C>` as-is for read-channel failures (per-Behavior); add a separate aggregate-Violation type for validate-channel failures. Decouples per-Behavior from aggregate.
- (d) Relocate non-trivial structural failures from Witness<C> into `Diagnostic` (which already carries SourceSpan + structured kind). `Witness<C>.Violates` stays simple (per-Behavior, string reason); rich structural failures accumulate at the Diagnostic level.

**Cascade implications:**
- (a): opaque-strings-attract-heuristics anti-pattern (per `feedback_opaque_strings_attract_heuristics`). Downstream consumers reading the reason string for programmatic filtering = bridge.
- (b): substrate change to dimensions.dag — affects all consumers of Witness<C> (existing AnalysisDimension, lens framework, future analyzers). Reverse cascade.
- (c): introduces parallel-representation between read-channel and validate-channel failure carriers. The current design has `read: Witness<C>` and `validate: OptionalDiagnostic` — already two channels. Adding a third type for structural validate failures is more parallel rep.
- (d): keeps Witness<C> simple; pushes structural failure data into lens-local `Diagnostic.kind` declarations.

**TestClaim shape** (renamed 2026-04-28 per gpt-5-5-pro BLOCKING re P2 single-authority — TestClaim names must reflect the locked decision that rich structural payloads live in `Diagnostic.kind`, NOT in `Witness<C>`):
- `diagnostic_kind_for_tenant_flow_carries_missing_capabilities_structurally` (verifies set-difference is recoverable from `Diagnostic.kind = CapabilityViolation { required, granted, missing }`, not from `Witness.reason: String`)
- `diagnostic_kind_for_ifc_carries_label_comparison_structurally` (verifies lattice-comparison is recoverable from `Diagnostic.kind = IFCDowngradeViolation { computed, sink_clearance, downgrade_required }`, not from `Witness.reason: String`)
- `no_string_parsing_in_witness_consumers` (anti-bridge — no consumer reads `Witness.reason: String` programmatically; structural payloads live exclusively in lens-local `Diagnostic.kind` declarations per the locked decision below)

**Recommendation:** **(d)** — `Witness<C>` stays as-is for per-Behavior read-channel failures (string reason is fine for human-readable per-node error messages); structural validate-channel failures encode into lens-local `Diagnostic.kind` declarations. Lens instances that need rich structural failure data (tenant-flow, IFC) declare their own kinds beside the lens instance (e.g., `CapabilityViolation { required, granted, missing }`, `IFCDowngradeViolation { computed, sink_clearance }`). This matches how the worked instances already encode validate failures (per the §"Three worked instances" section above). Witness<C> doesn't need extension.

**DECISION (Director-locked 2026-04-28 via dialogue; refined 2026-04-29): (c)/(d) hybrid — Witness<C> stays as-is; rich structural validation failures encode into lens-local `Diagnostic.kind` declarations via the lens-framework's structural inhabitance (uniform whether authored by compiler team or user lens — the compiler IS a user of its own system per `feedback_groundedness_gates_lenses`).**

Director's framing refinements:

1. **User/system distinction is artificial.** The compiler uses lenses to enforce violations the same way user-authored lenses do. `CapabilityViolation` as a lens-local diagnostic-kind declaration is uniform whether the lens that emits it is `dsl/std/lenses/cost.dag` (compiler-authored) or `user/myproject/lenses/tenant.dag` (user-authored). The kind extension surface is the *same* mechanism for both. INVARIANTS.md P1 procedure Step 3 reflects this — "lens-extensible" replaces the "user-extensible" framing.

2. **`Witness<C>` is one encoding of a deeper pattern (arity over an error space).** Pass = empty error list (zero arity); Fail = non-empty error list. Other valid encodings exist (`List<Error>` with empty = pass; `Maybe<Error>`; `(Result, List<Error>)`). The substrate's existing carriers (Witness<C>, DimensionReport<C>, OptionalDiagnostic) are domain-specific instances of this pattern. **No consolidation at this level** — the carriers work; recognizing the deeper pattern is observational, not actionable. Future authors of new decision/result carriers should follow the same shape (two disjoint variants, one for success-with-data, one for failure-with-context) but no substrate-level abstract `Either<A, B>` is needed.

**Substrate change required (Q6 framing — SUPERSEDED by Q6.5 below; see §Q6.5 for the refined disposition):** the Q6 lock framed extensions as "no substrate change" because `Witness<C>` stays at `dimensions.dag:35-37` and `CompilerDiagnosticKind` extensions were assumed to land per-lens during PR-K. Q6.5 refines this: the carrier surface DOES need a minimal additive widening of `Diagnostic.kind` to accept the two-layer parent — `CompilerDiagnosticKind`'s closed sum stays untouched, but `Diagnostic.kind`'s field type widens to `AnyDiagnosticKind` so it can carry both Layer-1 and Layer-2 kinds. Read this Q6 paragraph as historical framing; the live disposition is Q6.5.

### Q6.5 — Two-layer authority for diagnostic kinds (cross-manager protocol)

**Status:** Director-authored 2026-04-29 to operationalize Q6's "extends Diagnostic.kind via lens-framework's structural inhabitance" framing. Refines Q6 by naming the cross-manager handoff explicitly.

**Question Q6 left under-specified:** when an Evaluator lens-instance `validate` function needs a new diagnostic kind, who owns the addition? Substrate (closed sum at `src/v3/std/diagnostics.dag`)? Evaluator? Lens author? Q6's "lens-framework's structural inhabitance" phrase implies a mechanism but doesn't name the cross-manager surface.

**Resolution: two-layer diagnostic-kind authority.** Per INVARIANTS.md §P1 substrate-fact-introduction procedure Step 3 (Primitive-vs-lens-extensible check), diagnostic kinds split into two layers:

**Layer 1 — compiler-primitive diagnostic kinds.** Closed sum at `src/v3/std/diagnostics.dag` (`CompilerDiagnosticKind` + `DiagnosticKind` per `src/v3/std/verification.dag`). Owned by Substrate Manager. Variants: `TokenizerError`, `ParseError`, `TypeMismatch`, `UnitMismatch`, `ArityMismatch`, `ResolveError`, `NominalOpacityViolation` (current set). Adding a primitive kind = Substrate Manager edits the closed sum; rare, design-locked, requires SCAFFOLD-note acknowledgment of where the diagnostic surfaces.

**Layer 2 — lens-instance diagnostic kinds.** Declared **per lens** in the lens's own `.dag` via the lens framework's structural inhabitance pattern. Namespace-scoped to the lens. Examples: `CapabilityViolation` lives in `Lens<TenantFlow>`'s `.dag`; `IFCDowngradeViolation` lives in `Lens<IFC>`'s `.dag`; cost-lens-specific kinds (e.g., `BoundOverflow`) live in `Lens<SymbolicCost>`'s `.dag`. **Not variants of `CompilerDiagnosticKind`** — separate authorities, lens-namespace-scoped.

**Cross-manager handoff:**
- **Adding a Layer-1 kind:** Substrate Manager authors via PR to `src/v3/std/diagnostics.dag`. Cross-program coordination only if the kind affects existing consumers (rare; primitive kinds are stable).
- **Adding a Layer-2 kind:** lens author (Evaluator Manager for compiler-team lenses; arbitrary user for user-authored lenses) authors via PR to the lens's own `.dag`. **No Substrate Manager handoff required** — the kind is namespace-scoped to the lens.

**Runtime resolution:** when running `Lens<C>.validate(dag, c) → OptionalDiagnostic`, the runtime looks up diagnostic-kind authority via: (1) primitive lookup against `CompilerDiagnosticKind` closed sum; (2) lens-namespace lookup against the lens instance's declared kinds. Union semantics: a `Diagnostic` can carry either a Layer-1 kind or a Layer-2 kind; consumers dispatch via lens-context.

**Anti-bridge invariant:** lens authors MUST NOT extend `CompilerDiagnosticKind` for lens-instance kinds. Doing so would re-introduce the cross-manager handoff Q6.5 dissolves and would violate P2 single-authority (lens-instance kind authority lives in the lens, not in the substrate). The lens framework's structural inhabitance is the structural surface for Layer-2; modifying the closed sum is reserved for Layer-1 only. (Per `feedback_groundedness_gates_lenses.md`: language vocabulary is primitives + namespacing; the lens framework's inhabitance IS the namespacing for Layer-2.) Dynamic resolution by string interning is rejected per `feedback_opaque_strings_attract_heuristics.md`.

**Name-collision protocol (anti-shadowing):** a Layer-2 lens-instance kind MUST NOT reuse a Layer-1 `CompilerDiagnosticKind` variant name. A lens declaring `TypeMismatch` (Layer-1 variant) is a **compile error**, not silent shadowing. Structural separation between layers requires distinct namespaces; reuse defeats the layer separation by allowing a lens-namespace lookup to mask a substrate-primitive concern. Lens authors are structurally constrained to non-overlapping names; collision detection lives at the lens-framework's substrate-fact-introduction step (per INVARIANTS.md §P1 Step 3 — name uniqueness across both layers is verifiable structurally).

**Worked example: `Lens<TenantFlow>` declares `CapabilityViolation`:**

```
// dsl/std/lenses/tenant_flow.dag (or user-authored lens at any path)
inhabits Lens<TenantFlow> {
  diagnostic_kinds: [
    CapabilityViolation { required: CapSet, granted: CapSet, missing: CapSet },
    TenantBoundaryCrossed { from: TenantId, to: TenantId, declared_grants: CapSet }
  ]
  // ... read / sequential / branch / iterate / validate ...
}
```

The runtime sees `Lens<TenantFlow>.validate` produce a `Diagnostic { kind: CapabilityViolation { ... }, span, ... }` — kind looks up via lens-namespace, NOT against `CompilerDiagnosticKind`. The substrate's closed sum stays untouched.

**TestClaim shape (extends Q6's three claims):**
- `lens_instance_diagnostic_kind_namespace_scoped` — verifies a lens instance's diagnostic kinds resolve via the lens's own `.dag`, NOT via `src/v3/std/diagnostics.dag` `CompilerDiagnosticKind`
- `compiler_primitive_diagnostic_kind_closed_sum_unchanged_by_lens_instance_authoring` — verifies adding a Layer-2 kind doesn't touch `CompilerDiagnosticKind` (anti-bridge enforcement)
- `runtime_resolves_diagnostic_kind_via_two_layer_lookup` — verifies runtime dispatch falls through Layer-1 then Layer-2 correctly when looking up a kind by name

**DECISION (Director-authored 2026-04-29):** two-layer authority locked. Layer 1 = `CompilerDiagnosticKind` closed sum (Substrate-owned). Layer 2 = lens-instance kinds declared in the lens's own `.dag` via structural inhabitance (lens-author-owned, namespace-scoped). No cross-manager runtime handoff; no closed-sum churn for lens-instance authoring.

**Substrate change required: minimal additive widening of `Diagnostic.kind` to accept the two-layer parent; `CompilerDiagnosticKind` (Layer 1) closed sum stays untouched.** Concretely, `src/v3/std/diagnostics.dag:Diagnostic.kind` is currently strictly typed `kind: CompilerDiagnosticKind` (closed sum, Layer 1 only). To carry Layer-2 lens-instance kinds in the same `Diagnostic` value (which is what `Lens<C>.validate → OptionalDiagnostic` produces per `dimensions.dag:41-43`), the field type widens additively:

```
// Layer 2 lens-instance kind declaration shape (declared via lens framework's structural inhabitance).
//
// Owner is implied by structural containment: a DiagnosticKindDecl appears inside the `diagnostic_kinds`
// list of an `inhabits Lens<C>` declaration, and that containing inhabitance IS the owning lens. We
// deliberately do NOT carry `lens_instance: LensRef` as a duplicate field — that would re-introduce the
// parallel-authority illegal state (decl could appear under `Lens<TenantFlow>` while declaring
// `lens_instance: Lens<IFC>`). Instead, the owning lens is recovered structurally via the parent
// declaration of the decl in the program DAG (the enclosing `inhabits Lens<C>` block).
type DiagnosticKindDecl {
  name: KindName     // kind name; MUST NOT collide with Layer-1 CompilerDiagnosticKind variant names per anti-shadowing protocol above
  payload: TypeShape // payload type for this kind (e.g., `{ required: CapSet, granted: CapSet, missing: CapSet }`)
}

// Layer 2 carrier: a structural pointer to a declared DiagnosticKindDecl + payload checked to inhabit
// the decl's declared payload type. The two coordinates (decl + payload) are dependent, not independent —
// the lens framework's existing structural inhabitance check enforces that `payload` inhabits the
// `TypeShape` named by `kind_decl.payload`. This is the same mechanism that enforces `inhabits Lens<C>`
// claims at the lens-instance authoring boundary; we reuse it here so a `Diagnostic` cannot carry a
// payload whose shape disagrees with its declared kind.
type LensInstanceKindWitness {
  kind_decl: DeclarationRef     // resolves to a DiagnosticKindDecl whose parent declaration is the owning Lens<C> inhabitance
  payload: <inhabits kind_decl.payload>  // structural inhabitance witness — compiler-checked at construction
}

type AnyDiagnosticKind
  = CompilerKind(CompilerDiagnosticKind)        // Layer 1: substrate-owned closed sum (UNCHANGED)
  | LensInstanceKind(LensInstanceKindWitness)   // Layer 2: lens-namespace-scoped, dependent-typed

type Diagnostic {
  kind: AnyDiagnosticKind   // widened from CompilerDiagnosticKind to AnyDiagnosticKind
  span: SourceSpan
  message: String
  fixes: List<Correction>
}
```

The widening is ~5-15 lines in `diagnostics.dag`; `CompilerDiagnosticKind`'s closed sum stays as-is (Layer 1 unchanged). The anti-bridge invariant is preserved because Layer-2 kinds enter via the `LensInstanceKind` constructor through `LensInstanceKindWitness` — a structural pointer to a declared `DiagnosticKindDecl` + payload value compiler-checked to inhabit the decl's declared shape — not by extending `CompilerDiagnosticKind`.

**State-space discipline (per `feedback_state_space_vs_behavioral_invariants.md`).** A flat `LensInstanceKind(LensRef, KindName, KindPayload)` carrier (three independent coordinates) is rejected — it admits illegal states like `(Lens<TenantFlow>, "WrongName", payload-of-different-shape)`. The witness shape collapses kind name + payload into one structural reference (`DeclarationRef → DiagnosticKindDecl`) plus a compiler-checked inhabitance witness over that decl's declared payload. Two dependent coordinates, not three independent ones. The kind name recovers by projection from `kind_decl.name`; the owning `Lens<C>` recovers by parent-declaration navigation from the decl to its enclosing `inhabits Lens<C>` block (NOT from a duplicated `lens_instance` field on the decl — keeping ownership structurally implied by containment is what makes the namespace authority single, per `feedback_declare_facts_dont_derive.md` discipline applied here in reverse: do NOT re-declare what containment already says).

Rejected alternative: lens-instance validates returning a different carrier (breaks `OptionalDiagnostic` shape and lens framework spec).

**Note on existing SCAFFOLD:** `src/v3/std/verification.dag:49-62` has a 🟡 SCAFFOLD `DiagnosticKind` that mirrors `CompilerDiagnosticKind` for verification-runner consumption. That SCAFFOLD's dissolution trigger (per its own comment: "replace this mirror with typed references once substrate diagnostics are surfaced as declaration-shaped facts") is **separate** from Q6.5 — it tracks Layer-1 mirror retirement, not Layer-2 carrier authoring. Q6.5's `AnyDiagnosticKind` widening doesn't affect that SCAFFOLD's dissolution path.

**Cascade:**
- T-Substrate-Lens-Primitive dispatch consumes Q6.5: `Lens<C>` instance shape includes `diagnostic_kinds: List<DiagnosticKindDecl>` field via inhabitance + `Diagnostic.kind` widening lands as part of substrate authoring.
- R2-Evaluator's PR-A through PR-E consume Q6.5 unchanged: when authoring lens-instance `validate` functions, declare diagnostic kinds in the lens's `.dag`, not in Substrate's closed sum. Validate result populates `LensInstanceKind` constructor, not `CompilerKind`.
- R3-T-CostLens-Composition consumes Q6.5: `Lens<SymbolicCost>` instance declares its own kinds (`BoundOverflow`, etc.) in `dsl/std/lenses/cost.dag`, namespace-scoped via `DiagnosticKindDecl`.

### Q7 — Error-recovery semantics for partial validate failure

**Status:** REFERENCED in §"Open design questions" item 2. "Director's 'no silent fabrication' rule says report all" — but the spec doesn't actually state report-all semantics.

**Question:** when a program partially violates a side-condition (some paths leak; others don't), does `validate` report all violations or stop at the first?

**Alternatives:**
- (a) **Report-all**: the fold collects every validate failure across the DAG and emits a `DimensionFail` carrying ALL of them in `violations: List<Diagnostic>`. Implementer has to traverse the whole DAG even after first failure.
- (b) **Short-circuit**: the fold stops at the first validate failure; `DimensionFail.violations` carries one Diagnostic. Faster but loses information.
- (c) **Configurable**: lens declaration includes `failure_mode: ReportAll | ShortCircuit`. Each lens instance picks. Surface area increases.
- (d) **Per-call-site report-all, single aggregate**: each individual `validate` call returns at most one `OptionalDiagnostic` (the current shape); the fold collects them into the aggregate `violations: List<Diagnostic>`. So no SINGLE validate gives a list — but the DimensionFail accumulates them.

**Cascade implications:**
- (a) requires fold infrastructure to traverse the full DAG and accumulate failures; implementer can't early-terminate.
- (b) loses information that may be needed for diagnostic UX (the user wants ALL their tenant violations, not just the first one).
- (c) per-instance configuration adds complexity; deferring the choice to lens authors splits the discipline.
- (d) matches the current `validate: (Dag, C) → OptionalDiagnostic` signature. Each validate call yields zero or one diagnostic; the fold accumulates.

**TestClaim shape:**
- `validate_yields_at_most_one_diagnostic_per_call`
- `dimensionfail_violations_accumulate_across_dag` (the fold collects)
- `lens_does_not_silently_drop_violations` (no fabrication)
- `report_all_test_program_with_two_independent_violations` (cross-validation: program with two distinct tenant violations produces two Diagnostics in violations)

**Recommendation:** **(d)** — matches the current signature exactly. Each `validate` call returns one `OptionalDiagnostic`; the fold accumulates `SomeDiagnostic { value }` results into `DimensionFail.violations: List<Diagnostic>`. No spec change needed; the discipline is "fold accumulates; validate stays per-call." Practical effect: a program with two tenant-flow violations on different sub-DAGs yields two Diagnostics in `violations`. This satisfies "report all" without changing the validate signature or introducing per-instance configuration.

**DECISION (Director-locked 2026-04-28 via dialogue): (d).** No substrate change. The arity-over-error-space pattern (Director observation per Q6 dialogue) makes "report-all" fall out by construction — the fold's accumulation already gives empty list = pass, many entries = many failures. No special "report all vs short-circuit" decision; the existing signature does it.

### Q8 — Side-condition composition with mixed presence

**Status:** REFERENCED in §"Open design questions" item 5. `Lens<SymbolicCost> × Lens<SecurityLabel>` — complexity has no validate (always `NoDiagnostic`); IFC has validate. Cross-product `Lens<(SymbolicCost, SecurityLabel)>` — what's the validate behavior?

**Question:** when forming the cross-product `Lens<C> × Lens<D>` and one of them has no side-condition, what's the cross-product's validate semantics?

**Alternatives:**
- (a) **Conjunctive**: cross-product validate is `validate_C(c) ∧ validate_D(d)`. If either fails, result is failure. NoDiagnostic from one side defaults to "passed" for that conjunct. (Standard logical conjunction.)
- (b) **Per-side independent**: cross-product validate runs both; combined result is `DimensionFail` if EITHER fails. Same outcome as (a) but framed differently.
- (c) **Side-eject**: lenses without validate eject from the cross-product; cross-product validate is just the validate of the lens that has one. (Loses symmetry.)
- (d) **Validate-required**: cross-product requires BOTH lenses to have validate. NoDiagnostic-only lenses can't compose. (Restrictive.)

**Cascade implications:**
- (a): standard semantics; matches `feedback_modeling_philosophy` (compose facts forward; conjunction is the natural product). No extra surface.
- (b): same outcome as (a) operationally; just different framing.
- (c) breaks the product symmetry — `Lens<C> × Lens<D>` ≠ `Lens<D> × Lens<C>` when only one has validate.
- (d) excludes complexity from cross-products; loses the "complexity × IFC" use case the design wants.

**TestClaim shape:**
- `cross_product_validate_is_conjunction`
- `cross_product_with_no_validate_side_treats_as_nodiagnostic`
- `complexity_x_ifc_validates_ifc_when_complexity_has_no_validate` (the worked case from D5)
- `cross_product_symmetric_under_lens_swap` (verifies a × b ≡ b × a in terms of validate outcome)

**Recommendation:** **(a)** — conjunctive. Cross-product validate = `validate_C(dag, c) ∧ validate_D(dag, d)` where NoDiagnostic acts as logical TRUE. Result is `DimensionFail` if either is `SomeDiagnostic`; `DimensionOk` if both are `NoDiagnostic`. This is the standard product-monoid semantic and preserves cross-product symmetry. Implements via straightforward fold-time conjunction.

**DECISION (Director-locked 2026-04-28 via dialogue): (a).** No substrate change. Cross-product validate runs both validates; accumulates via Q7's fold (`SomeDiagnostic.value` lifts into `DimensionFail.violations: List<Diagnostic>`). Conjunctive logic falls out of the existing arity-over-error-space encoding — empty list = pass; non-empty = fail. NoDiagnostic-only lenses (complexity) compose freely with validate-having lenses (IFC); cross-product preserves both contributions.

### Pre-dispatch design-PR cadence (lens framework)

| PR | Locks | Before dispatch of | TestClaim gate |
|---|---|---|---|
| **PR-K** | Witness<C> generality decision (Q6) + error-recovery semantics (Q7) + cross-product composition (Q8) | T-Substrate-Lens-Primitive | All Q6 + Q7 + Q8 TestClaims pass |

Single cadence PR; the three questions are tightly coupled (all about lens-framework spec semantics) and benefit from being decided together. Director signoff before T-Substrate-Lens-Primitive dispatch.

## Cross-refs

- Parent: [`docs/design-emission-model.md`](design-emission-model.md) §"Modeling problem 8 — cost lens over emission" + §"Open call 4 — Lens-as-parametric-monoid framework"
- Substrate lane: [`docs/r2-structure.md`](r2-structure.md) §"Substrate Manager" T-Substrate-Lens-Primitive sub-lane
- Consumer lanes: [`docs/r3-structure.md`](r3-structure.md) T-CostLens-Composition + T-Verification-L4-L7-Direct
- Existing primitives this framework consumes: `src/v3/std/dimensions.dag` (`Witness<Carrier>` + `DimensionReport<Carrier>`); `dsl/std/algebra.dag` (`Lattice<T>`, `BoundedLattice<T>`)
- Existing lenses to migrate: `src/v3/lenses/{cost,complexity,idempotency,parallelism}.dag`
- THESIS: [`THESIS.md`](../THESIS.md) §"User-defined dimensions" + §"Concept unification" (coercion cost = complexity) + §"Free consequences"
