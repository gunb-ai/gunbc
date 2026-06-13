# Measure value-typing — design (scope / dual-representation dissolution)

**Status:** DESIGN / SCOPING ONLY — no substrate lands from this doc until reviewed.
**Work item:** `node://adhoc-4523b7d2-967`
**Session:** zesty-otter-413 (compute-fabric subtree)
**Parent:** snappy-stag-903 (compute fabric)
**Authority:** `dsl/std/measure.dag` (Q-Unit-1..5 + option (c) ratified at gunbc#828)

## 1. Problem — dual representations at the compute-fabric boundary

`Measure<Q, S>` is ratified as a **phantom-parameter dimensional carrier** with **no runtime
payload** (`measure.dag:48–54`, `measure.dag:176`). That was correct Phase-1 carrier landing.
Downstream compute-fabric consumers now carry the same facts twice:

| Fact | Typed authority (phantom) | Parallel bare `Int` authority |
|------|---------------------------|-------------------------------|
| RAM bytes | `ByteSize = Measure<Memory, One>` on `MemoryFacts.capacity`, `MemoryRequirement.min_bytes`, … | `PlacementSupplyRow.ram_bytes`, `HostRamSupplyFacts.observed_usable_bytes`, `placement_supply_row(ram_bytes_total: Int)` |
| Clock rate | `Hertz = Measure<Frequency, One>` (alias only) | `CpuDeploymentFacts.sustained_per_thread_hz`, `PlacementSupplyRow.clock_hz` |
| Hardware threads | *(no alias yet)* | `PlacementSupplyRow.hardware_threads`, `cpu_facts_threads` projection |
| Price | `Quantity::Currency` in `measure.dag`; `CostEstimate.currency: NonEmptyStr` | `CostEstimate.expected_units: Float` (untyped magnitude + stringly currency) |

The explicit debt marker is in `compute_fabric.dag:198–200`:

> `ram_bytes_total` is an explicit projection parameter because `ByteSize` unwrap is not yet
> value-typed on the v2 interpreter; dissolve when Measure carriers gain inhabitants.

This is a **P2 Boundary Discipline** violation (INVARIANTS.md): the dimensional tag and the bare
integer coexist as parallel authorities for the same fact. The fix is not more projection
parameters — it is making `Measure` carriers **value-inhabitable** so consumers hold one authority.

## 2. Ratification constraints (gunbc#828 / Q-Unit)

These are **hard boundaries** for any value-typing follow-up; do not re-litigate in implementation
PRs:

| Decision | Source | Constraint on value-typing |
|----------|--------|---------------------------|
| Q-Unit-1 | Measure rename from `Unit`; `One` scale identity | Keep `Measure` outer name; `One` = 10⁰ |
| Q-Unit-4 | Outer `Refined` / inner `Measure`; **option (c)** ratified | **No** `Refined<Measure<…>, predicate>` at this layer. Non-negativity / range predicates are a downstream concern when `Refined<>` consumes a base with numeric inhabitants. |
| Q-Unit-4 | Grounding owns per-target representation | Value body is substrate-neutral; Rust `u64` bytes, Go `int64`, etc. are Grounding projections — not parallel `.dag` authorities. |
| Q-Unit-5 | Quantity / Scale dissolution trigger | New `Quantity` / `Scale` values still require ≥1 Grounding emission-rule consumer. Value-typing does **not** widen Quantity / Scale enums. |
| Constrained inhabitance gap | `measure.dag:149–169` | `<Q> : Quantity, <S> : Scale` where-clauses are a **substrate-feature lane** — defer mechanical witness derivation; sole consumers instantiate ratified pairs by construction. |

**Option (c) posture preserved:** value-typing adds a **numeric magnitude field** inside
`Measure<Q, S>`; it does **not** add predicate refinement at the Measure level.

## 3. Proposed model — one value body, phantom tags unchanged

### 3.1 Core carrier

Widen `Measure<Q, S>` from an opaque phantom atom to a **single-field record** whose field
asserts the one fact all dimensional magnitudes share at this layer: *how much, in the declared
scale*.

```dag
// dsl/std/measure.dag — proposed shape (not yet landed)
type Measure<Q, S> {
  count: Int
}
```

**Field choice `count: Int` (not `Nat`) for Phase-1:**

- `placement_supply.dag` is intentionally **v2-importable** with only `std.types` (PR #4686) —
  it must not transitively load `nat.dag` → `algebra.dag` → `magnitude.dag` until that chain is
  verified v2-green.
- `Int` is the counting authority already used at every dual-representation site today.
- **Forward flag:** when the `Magnitude → Nat` chain is v2-importable (T-Numeric-Construction
  cascade), migrate `count: Int` → `count: Nat` in one PR with no consumer field renames (same
  label). `Nanoseconds { count: Nat }` in `v3.std.timing_lens` is the v3-substrate precedent.

**Phantom discipline:** `<Q>` and `<S>` remain type parameters tagging dimensional semantics.
They do not appear in the record. `ByteSize { count: 99857989632 }` is memory-at-scale-`One`;
the `Memory` / `One` tags are static.

**Scale interpretation:** the stored `count` is in the units implied by the `(Q, S)` pair —
e.g. `Measure<Time, Milli>` stores milliseconds, `Measure<Memory, One>` stores bytes.
`scale_exponent` remains the single authority for cross-scale conversion when that consumer
lands; not in Phase-1.

### 3.2 Phase-1 dimensional aliases

Land alongside the value body in `measure.dag`:

| Alias | Definition | Replaces bare `Int` at |
|-------|------------|------------------------|
| `ByteSize` | `Measure<Memory, One>` | RAM fields (placement, memory facts, requirements) |
| `Hertz` | `Measure<Frequency, One>` | `sustained_per_thread_hz`, `clock_hz` |
| `HardwareThreadCount` | `Measure<Count, One>` | `hardware_threads` (new named concept — distinguishes schedulable hardware threads from generic counts) |
| `MoneyAmount<S>` | `Measure<Currency, S>` | `CostEstimate` magnitude (Phase-3; see §5.3) |

`Bandwidth`, `Duration` aliases already exist on `compute_fabric.dag`; they inherit the value
body automatically once `Measure` is widened.

### 3.3 Constructors and projections (per-alias, not generic)

The substrate has **no parametric where-clauses** and v2 may not support generic
`fn measure<Q,S>(count: Int) -> Measure<Q, S>`. Phase-1 uses **named total functions per
alias** (M9: attach to existing `scale_exponent` / measure authority, don't hand-roll per-site):

```dag
fn byte_size(count: Int) -> ByteSize
fn hertz(count: Int) -> Hertz
fn hardware_thread_count(count: Int) -> HardwareThreadCount
```

Projections for arithmetic that must stay v2-safe:

```dag
fn byte_size_count(b: ByteSize) -> Int
fn hertz_count(h: Hertz) -> Int
fn hardware_thread_count_value(t: HardwareThreadCount) -> Int
```

**Dissolution rule:** projections exist only until consumers hold the typed value end-to-end.
`placement_supply_row` loses the `ram_bytes_total: Int` parameter when
`supply_srv*_ram.observed_usable_bytes` becomes `ByteSize` (§4).

## 4. Dual-representation dissolution map

Ordered by consumer demand (compute-fabric / placement first):

### 4.1 `std.placement_supply` (v2-safe — highest priority)

| Before | After |
|--------|-------|
| `hardware_threads: Int` | `hardware_threads: HardwareThreadCount` |
| `clock_hz: Int` | `clock_hz: Hertz` |
| `ram_bytes: Int` | `ram_bytes: ByteSize` |
| `cpu_capacity_hz_row` multiplies bare ints | multiply via `hertz_count` × `hardware_thread_count_value` **or** add typed `cpu_capacity_hz_row` overload returning `Hertz` (prefer typed product alias `Hertz` at `Measure<Frequency, One>` — capacity = threads × per-thread rate is a derived `Hertz`) |

Import graph constraint: `placement_supply.dag` may import `std.measure` (currently **zero**
transitive imports — safe) but must **not** import `compute_fabric` or `float.dag`.

### 4.2 `std.compute_fabric` projection

| Before | After |
|--------|-------|
| `placement_supply_row(identity, cpu, ram_bytes_total: Int)` | `placement_supply_row(identity, cpu, ram: ByteSize)` |
| `cpu_facts_sustained_per_thread_hz(cpu) -> Int` used at projection | `-> Hertz`; `CpuDeploymentFacts.sustained_per_thread_hz: Hertz` |
| `supply_srv*_placement_row` passes `observed_usable_bytes` int | passes `byte_size(supply_srv*_ram.observed_usable_bytes)` after §4.3 |

### 4.3 `std.memory` / `std.cpu`

| Before | After |
|--------|-------|
| `HostRamSupplyFacts { nominal_bytes: Int, observed_usable_bytes: Int }` | both `ByteSize` |
| `CpuDeploymentFacts.sustained_per_thread_hz: Int` | `Hertz` |
| `DramModuleCatalogRow.capacity_bytes: Int` | `ByteSize` (catalog stick capacity — same fact) |
| `host_memory_population_nominal_bytes -> Int` | `-> ByteSize` |

### 4.4 `CostEstimate` (Phase-3 — needs scale decision)

Current: `{ model, expected_units: Float, currency: NonEmptyStr }`.

Target: `{ model, amount: MoneyAmount<S>, currency: CurrencyCode }` where `CurrencyCode` is a
branded `NonEmptyStr` (or closed enum when ISO-4217 rows land).

**Open design point (needs director ratification before implementation):**

- SI `Scale` on `Currency` is not financially standard. Options:
  - **(a) Minor units:** `MoneyAmount<One>` stores integer minor units (cents); currency code
    disambiguates exponent (USD cents vs JPY whole-yen). Document per-currency minor-unit
    exponent in catalog rows (extdeps), not in the `Scale` tag.
  - **(b) Major units + deferred fraction:** `MoneyAmount<One>` stores whole units; fractional
    prices wait for `Rational` / `Real` consumer (do not use `Float` as permanent authority).
  - **(c) Defer price** to Phase-3 after ByteSize/Hertz/threads land; no partial dissolution.

**Recommendation:** **(c) for Phase-1/2**, **(a) for Phase-3** — integer minor units align
with `count: Int`, avoid `Float` dual authority, and match billing-system facts.

## 5. Phased implementation

| Phase | Scope | Consumer validated by |
|-------|-------|----------------------|
| **0** | This design doc + director review | — |
| **1** | `Measure<Q,S>` value body + aliases + constructors/projections in `measure.dag` | v2 compile of `measure.dag`; unit claim that `ByteSize { count: N }` inhabits |
| **2** | `placement_supply` + `compute_fabric` migration (§4.1–4.2) | v2 compile of `placement_supply.dag` without `float.dag`; existing compute_fabric witnesses still parse |
| **3** | `memory` + `cpu` migration (§4.3) | `supply_srv{1,2}_*` data rows type-check; ctrl plan-models pin can drop `ram_bytes_total` int param |
| **4** | `CostEstimate` / currency (§4.4) | gated on Phase-3 + director scale decision |
| **5** | `Nanoseconds` → `Measure<Time, Nano>` alignment in `v3.std.timing_lens` | coordinate with timing-lens lane; **not** in compute-fabric worker scope |

Each phase is **one PR** per dispatch discipline. Phases 1–2 can bundle if ratchet stays green.

## 6. Substrate gaps and escalation triggers

| Gap | Blocks? | Action |
|-----|---------|--------|
| Parametric where-clauses `<Q: Quantity, S: Scale>` | No (Phase-1) | Existing debt; consumers use ratified instantiations only |
| v2 value construction for `Measure<Memory, One> { count: … }` | **Maybe** | **Measure in Phase-1** — if v2 cannot construct parametric record values, escalate to substrate manager for v2 literal / data-body support before Phase-2 |
| `Nat` import chain on v2 | No (Phase-1) | Stay on `Int`; migrate field type later |
| Grounding emission rules for new Quantity consumers | No | Q-Unit-5: no new Quantity/Scale values in this lane |
| `Refined<Measure<…>, non_negative>` | No (option c) | Explicitly out of scope |

**Escalate** via `dashboard-ops escalate` if Phase-1 spike shows v2 cannot inhabit
`ByteSize { count: N }` in `data` rows — that is a substrate blocker, not a modeling workaround
(no permanent `Int` fallback fields).

## 7. Out of scope

- Aspect axis (PointKind / Magnitude / Instant / Rate) — R4 C6
- Scale-agnostic `Duration<S>` parametric form — needs ≥2 consumers
- `Refined<Measure<…>, predicate>` wrapper — option (c) deferral stands
- Mechanical `Measure` witness derivation / inhabitance lens — substrate-feature lane
- `Nanoseconds` / timing-lens migration — Phase-5, separate owner
- Grounding emission rules (Rust `u64`, Go `int64`, …) — Grounding manager lane
- ctrl `plans/models/capacity` pin bump — follow-on after Phase-2 merges (ctrl#1545)

## 8. Acceptance criteria (implementation phases)

1. **No dual representation** at migrated sites: each fact has exactly one typed field; no parallel
   bare `Int` parameter threading the same magnitude.
2. **`placement_supply.dag` import graph** remains free of `float.dag` / `integer.dag`
   `MachineWidth` chain.
3. **Q-Unit option (c):** no predicate refinement added to `Measure` declaration.
4. **M2:** `Measure` authority remains solely in `dsl/std/measure.dag`; aliases are thin `=`
   definitions, not re-declarations.
5. **Consumer-by-execution:** at least one `data` row (`supply_srv1_placement_row` or successor)
   constructs typed `ByteSize` / `Hertz` / `HardwareThreadCount` values and compiles on v2.

## 9. Test plan sketch

| Check | Phase |
|-------|-------|
| v2 compile: `dsl/std/measure.dag` | 1 |
| v2 compile: `dsl/std/placement_supply.dag` (isolated) | 2 |
| Structural ratchet: `Measure` declaration has exactly one field `count` | 1 |
| `placement_supply_row` arity: no `Int` ram parameter | 2 |
| ctrl integration: plan-models pin + capacity.dag import (manual, post-merge) | 2+ |

---

**Next action:** director / parent review of §4.4 Currency scale decision and §3.1 `Int` vs `Nat`
field choice. On approval, dispatch Phase-1 implementation worker (or continue in this session).
