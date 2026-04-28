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
  name:     String              // dimension identifier (populates DimensionReport.dimension_name)
  read:     Node → Witness<C>   // per-node cost basis (read from substrate facts; typed failure channel)
  unit:     C                   // identity element of C's monoid
  compose:  (C, C) → C          // sequential composition (Bind)
  parallel: (C, C) → C          // parallel composition (Branch)
  iterate:  (C, Bound) → C      // bounded iteration (Loop)
  validate: C → Witness<C>      // side-condition (optional; for fail-closed lenses)
}
```

**Why `read` returns `Witness<C>` and not `C`** (per codex BLOCKING finding on `4057b8f5`): a `Node → C` signature has no typed failure channel — a missing per-node substrate fact (e.g., a primitive without a declared cost) would have to either fabricate a default `C` (violates fail-closed) or panic (violates fail-closed). `Witness<C>` from `src/v3/std/dimensions.dag:35-37` is the existing typed channel:

```
type Witness<Carrier>
  = Inhabits(Carrier)              // fact present; carries the read value
  | Violates { reason: String, at: Behavior }   // fact missing; structured failure
```

The fold accumulates `Witness<C>` outputs from `read`. Any `Violates` becomes a `Diagnostic` in the final `DimensionFail.violations` — no fabricated carrier ever reaches `compose`. This matches the existing `AnalysisDimension.witness_of: fn(Dag, Behavior) -> Witness<Carrier>` pattern at `dimensions.dag:74`.

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

Each instance demonstrates a different monoid shape and validates the framework's range. All three share the same fold machinery; they differ only in (read, unit, compose, parallel, iterate, validate).

### Instance 1 — Complexity (additive numeric monoid)

Establishes the basic framework shape.

**Cost basis declaration:**
```
type SymbolicCost = CostExpr {
  work:              SymbolicExpr   // total work (sum of operations)
  span:              SymbolicExpr   // critical-path length (max of parallel arms)
  asymptotic_class:  BigOClass      // O(1), O(log n), O(n), O(n log n), O(n^2), ...
}
```

**Lens<SymbolicCost> declaration:**

| Field | Definition |
|---|---|
| `name` | `"complexity"` |
| `read(node)` | Reads operation's declared cost from `dsl/std/algebra.dag` and wraps as `Inhabits(...)` (e.g., `OrderedRing.add` → `Inhabits(CostExpr(1, 1, O(1)))`); `Violates { reason: "no declared cost for <op>", at: <node.behavior> }` if the substrate has no fact for that operation |
| `unit` | `CostExpr(work=0, span=0, asymptotic_class=O(1))` |
| `compose` (Bind) | `CostExpr(work=a.work + b.work, span=a.span + b.span, class=max(a.class, b.class))` |
| `parallel` (Branch) | `CostExpr(work=a.work + b.work, span=max(a.span, b.span), class=max(a.class, b.class))` |
| `iterate` (Loop) | `CostExpr(work=body.work × bound, span=body.span × bound, class=multiply_class(body.class, bound))` |
| `validate` | None (complexity has no side-condition; always `DimensionOk` if all reads `Inhabits`) |

**Worked program:**
```
data x = compute_pair(in)             // bind: O(n)
data y = parallel { sort(x), aggregate(x) }   // branch: max
data z = compute_summary(y, in)        // bind: O(1)
```

**Fold trace** (every `read` returns `Witness<CostExpr>`; the fold unwraps `Inhabits(c)` and accumulates witnesses; any `Violates` would short-circuit to `DimensionFail`):
1. `read(compute_pair)` → `Inhabits(CostExpr(n, n, O(n)))`
2. `read(sort)` → `Inhabits(CostExpr(n log n, log n, O(n log n)))`
3. `read(aggregate)` → `Inhabits(CostExpr(n, n, O(n)))`
4. `parallel(sort, aggregate)` → `CostExpr(n log n + n, max(log n, n), O(n log n))` (compose pulls inhabited values; same for steps 5/7)
5. `compose(compute_pair, parallel(...))` → `CostExpr(n + (n log n + n), n + max(log n, n), O(n log n))`
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
| `read(node)` | Reads operation's declared capability requirement set wrapped as `Inhabits(...)` (e.g., `read[TenantA].orders` → `Inhabits({Read(TenantA)})`); `Violates { reason: "no declared capability requirement for <op>", at: <node.behavior> }` if the substrate has no fact for that operation |
| `unit` | `{}` (empty capability set) |
| `compose` (Bind) | Set union — sequential composition accumulates capabilities |
| `parallel` (Branch) | Set union — parallel branches require all capabilities of any arm |
| `iterate` (Loop) | Set union of body × any-iter (bound doesn't matter for capability set; all iterations together) |
| `validate(set)` | Reject if program crosses tenant boundary not granted in declared workflow capability grant — emit `Diagnostic { kind: CapabilityViolation, message: "required: <set>, granted: <workflow.cap_grant>, missing: <set ∖ granted>", ... }` (the `CapabilityViolation` kind is a new `CompilerDiagnosticKind` variant landed alongside this lens instance) |

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
   - Missing: `{Write(TenantB)}` — **NOT GRANTED**
6. **Result:** `DimensionFail { dimension_name: "tenant-flow", violations: [Diagnostic { kind: CapabilityViolation, message: "required: {Read(TenantA), Write(TenantB)}, granted: {Read(TenantA), Write(TenantA)}, missing: {Write(TenantB)}", ... }], witnesses: [...] }`

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
| `read(node)` | Reads operation's data label wrapped as `Inhabits(...)` (e.g., `read[TopSecret].records` → `Inhabits(TopSecret)`); `Violates { reason: "no declared security label for <op>", at: <node.behavior> }` if the substrate has no fact for that operation |
| `unit` | `Public` (lattice bottom) |
| `compose` (Bind) | Lattice join (`max`) — sequential composition takes the highest label |
| `parallel` (Branch) | Lattice join (`max`) — parallel branches yield the highest label |
| `iterate` (Loop) | Lattice join (`max`) of body × any-iter (label doesn't change with bound) |
| `validate(label)` | Reject if program output label ⊐ sink's clearance — emit `Diagnostic { kind: IFCDowngradeViolation, message: "computed: <label>, sink_clearance: <sink.label>, downgrade_required: <label ⊐ sink.label>", ... }` (the `IFCDowngradeViolation` kind is a new `CompilerDiagnosticKind` variant landed alongside this lens instance) |

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
   - `TopSecret ⊐ Confidential` → downgrade required, NOT granted
7. **Result:** `DimensionFail { dimension_name: "ifc", violations: [Diagnostic { kind: IFCDowngradeViolation, message: "computed: TopSecret, sink_clearance: Confidential, downgrade_required: true", ... }], witnesses: [...] }`

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
| `src/v3/lenses/complexity.dag` | PROXY — single integer depth per port | `Lens<Depth>` instance with `Depth = Int` and `compose = max`, `parallel = max + 1` | ~1 day |
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
- For each of the 4 existing lenses, write the instance declaration (`read`, `unit`, `compose`, `parallel`, `iterate`, `validate`) using only existing combinators where possible.
- **Pass criterion:** every existing combinator (`combine_max`, `combine_sequential`, etc.) maps to a Lens<C> field with no machinery left over. Failure = the existing patterns aren't actually monoidal, or the framework needs additional fields.

**D3. L6 (structural-form coverage) collapses to `Lens<EmissionPathPresent>`.**
- Write L6 as a `Lens<EmissionPathPresent>` instance:
  - `read(node)` reads the emission-path declaration for (substrate form × Shape A target) pair
  - `unit` = `present` (vacuous true)
  - `compose` = AND (all forms must be present)
  - `validate` = reject if any (form, target) lacks an emission path declaration
- **Pass criterion:** L6's spec from r3-structure.md acceptance gates is satisfied by this instance. Failure = L6 isn't actually a structural fold (Codex Pattern B finding was wrong).

**D4. Cross-domain composition works for the 3 instances.**
- Hypothetical: a program has both complexity claims AND IFC claims.
- Compose `Lens<SymbolicCost> × Lens<SecurityLabel> = Lens<(SymbolicCost, SecurityLabel)>`.
- Verify side-conditions compose conjunctively (both must satisfy).
- **Pass criterion:** the composed lens correctly identifies a program that satisfies complexity but violates IFC (or vice versa). Failure = product monoid spec is wrong.

**D5. DimensionReport<C> covers all failure modes.**
- Enumerate failure modes across the 3 instances:
  - **Read-channel failures** (every instance): substrate has no fact for a node — `read` returns `Violates { reason, at }`; fold short-circuits to `DimensionFail` with that violation surfaced as a `Diagnostic` (kind = `MissingSubstrateFact` or per-instance variant)
  - **Validate-channel failures** (per instance): complexity has none; tenant has `CapabilityViolation`; IFC has `IFCDowngradeViolation`
- Verify each maps to a `Diagnostic` value with an appropriate `CompilerDiagnosticKind` variant (lens instances may extend `CompilerDiagnosticKind` with their own kinds).
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
- Lens<C> declaration includes the 7 fields (`name`, `read`, `unit`, `compose`, `parallel`, `iterate`, `validate`).
- `read: Node → Witness<C>` (typed failure channel; reuses `Witness<Carrier>` from `src/v3/std/dimensions.dag:35-37`).
- Type-checks against existing `BoundedLattice<T>`, `DimensionReport<Carrier>`, and `Witness<Carrier>` patterns.
- **Pass criterion:** substrate parses; structural-form ratchet remains green; no fabricated-carrier path exists in `read` (codex BLOCKING discipline).

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

**I6. L6 reframe: structural-form-coverage as `Lens<EmissionPathPresent>` passes.**
- TestClaim `l6_via_lens_framework_passes`: L6's structural cross-product fold expressed as Lens<EmissionPathPresent> instance produces the same answer as a hand-coded fold.
- **Pass criterion:** L6 acceptance gate `l6_structural_form_coverage` passes via the framework.

**I7. User-authored lens TestClaim.**
- A user (test author) declares a custom `Lens<MyCostBasis>` with their own monoid; runs the fold; gets a result.
- TestClaim `user_authored_lens_via_framework`: confirms the user-lens surface uses the same machinery as built-ins.
- **Pass criterion:** no special path for user lenses vs built-ins.

**I8. Read-channel fail-closed TestClaim** (added per codex BLOCKING finding on `4057b8f5`).
- Construct a program that uses an operation for which the substrate has no declared cost/capability/label fact (whichever instance is being tested).
- Run the lens fold; expect `DimensionFail` with `violations` containing a `Diagnostic` whose kind names the missing fact and whose `at` field identifies the offending node/behavior.
- TestClaims:
  - `lens_complexity_missing_cost_fail_closed`: fold on a program with an op lacking declared cost → `DimensionFail`
  - `lens_tenant_flow_missing_cap_fail_closed`: fold on a program with an op lacking declared capability requirement → `DimensionFail`
  - `lens_ifc_missing_label_fail_closed`: fold on a program with an op lacking declared security label → `DimensionFail`
- **Pass criterion:** every instance's `read` returns `Witness<C>`; missing facts produce typed diagnostics; no fabricated `C` ever reaches `compose`.

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

3. **Lens-application performance.** The fold visits every Node in the DAG. For large programs, can we memoize? If so, on what key (Node identity? structural hash?). This is post-R3 optimization; flagging here so we don't accidentally bake in non-memoizable shape.

4. **Parametric algebra interaction.** `Dimension<Carrier>` (R2 Modeling) is a parametric type. Does a `Lens<Dimension<...>>` make sense? Should be expressible if `Lens<C>` is fully parametric.

5. **Side-condition composition when one lens has none.** Complexity has no side-condition; IFC has one. `Lens<SymbolicCost> × Lens<SecurityLabel>` should still validate IFC correctly. Verify D4 covers this.

These are research-questions for the design phase; not blockers for the substrate primitive itself but worth recording.

## Cross-refs

- Parent: [`docs/design-emission-model.md`](design-emission-model.md) §"Modeling problem 8 — cost lens over emission" + §"Open call 4 — Lens-as-parametric-monoid framework"
- Substrate lane: [`docs/r2-structure.md`](r2-structure.md) §"Substrate Manager" T-Substrate-Lens-Primitive sub-lane
- Consumer lanes: [`docs/r3-structure.md`](r3-structure.md) T-CostLens-Composition + T-Verification-L4-L7-Direct
- Existing primitives this framework consumes: `src/v3/std/dimensions.dag` (`Witness<Carrier>` + `DimensionReport<Carrier>`); `dsl/std/algebra.dag` (`Lattice<T>`, `BoundedLattice<T>`)
- Existing lenses to migrate: `src/v3/lenses/{cost,complexity,idempotency,parallelism}.dag`
- THESIS: [`THESIS.md`](../THESIS.md) §"User-defined dimensions" + §"Concept unification" (coercion cost = complexity) + §"Free consequences"
