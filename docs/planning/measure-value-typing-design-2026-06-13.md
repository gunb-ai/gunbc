# Measure value-typing — design (scope / dual-representation dissolution)

**Status:** DESIGN / SCOPING ONLY — feasibility spike complete 2026-06-13; **no kernel
lands until operator/manager confirms scope.**
**Work item:** `node://adhoc-4523b7d2-967`
**Session:** zesty-otter-413 · **Manager:** snappy-stag-903 (compute fabric)
**Authority:** `dsl/std/measure.dag` (Q-Unit-1..5 + option (c) ratified at gunbc#828)

---

## 0. Feasibility finding (BLOCKING — read first)

**Verdict: INTERPRETER/COMPILER-GATED for the v2 claim-run path; modeling-only for v3
bootstrap typecheck.**

The recurring mark *"not yet value-typed on the v2 interpreter"* is accurate. Three
independent v2 gaps were reproduced with `/tmp` spike modules (not committed):

| Gap | Symptom (v2) | Spike |
|-----|----------------|-------|
| **G1 — phantom type args** | `Measure<Memory, One>` → `unresolved type 'Memory'` / `'One'` | `gunbc run --entry dsl/std/measure.dag` fails today on alias lines 179/184 |
| **G2 — alias field access** | `ByteSize.count` → `no field 'count' on type 'ByteSize'` when `ByteSize = Box<Int>` and `Box { value: Int }` | infer `lookup_field_type_node` returns `None` for `NoConnective` alias nodes (does not chase to target record) |
| **G3 — parametric record literals** | `Box<Int> { value: 42 }` in `data` rows → `undefined variable 'Box'` | non-parametric `type ByteSize { count: Int }` **does** claim-run green |

**Control (v2 works):** non-parametric `type ByteSize { count: Int }` with `data` rows +
field access + `let` constructors — both witnesses return `true`.

**Implication:**

- **Kernel modeling** (widen `Measure<Q,S>` to `{ count: Int }`, add aliases/constructors) is
  **tractable for v3 bootstrap** — dsl/std already participates in v3 generated DAG; no spike
  blocker on the modeling shape itself.
- **Dissolving the "not yet value-typed on v2 interpreter" marks** requires a **v2
  substrate/interpreter lane** (G1–G3) before `placement_supply` witnesses can claim-run with
  typed fields. Do **not** fake this with parallel non-parametric record types (M2 / Q-Unit
  violation).
- **Recommended sequencing:** (A) land kernel model for v3 typecheck + ratchet; (B) escalate
  G1–G3; (C) migrate std consumers; (D) manager lands `compute_fabric.dag` after #4810.

---

## 1. Problem — dual representations

`Measure<Q, S>` is phantom-only (no runtime payload). Compute-fabric std deferrers hold the
same facts twice — typed alias + bare `Int`/`Float`. Explicit debt:
`compute_fabric.dag:198–200` (`ram_bytes_total: Int` projection parameter).

**Operator goal:** proper dimensioned models, **no dual representations** — numbers live
**inside** `ByteSize`, `Hertz`, `HardwareThreadCount`, `Measure<Currency, S>`.

---

## 2. Ratification constraints (gunbc#828 — do not re-litigate)

- **Option (c):** no `Refined<Measure<…>, predicate>` at this layer.
- **Phantom tags:** `<Q>` / `<S>` carry dimensional semantics; value body carries magnitude only.
- **Grounding:** per-target `u64` / `int64` etc. are projections, not parallel authorities.
- **Q-Unit-5:** do not add Quantity/Scale enum values in this lane.
- **Where-clauses** `<Q: Quantity, S: Scale>` — substrate-feature lane; consumers instantiate
  ratified pairs by construction.

---

## 3. Minimal value-typing design (kernel — `measure.dag`)

### 3.1 Widen carrier (ratification header updated, not replaced)

```dag
import std.types { Int }

type Measure<Q, S> {
  count: Int
}
```

- **`count: Int`** (not `Nat`) — matches every census `Int` site; v2-importable without
  `nat.dag` → `algebra.dag` chain. Forward-flag: migrate to `Nat` when construction chain is
  v2-green (same field label).
- Phantom `<Q>` / `<S>` unchanged. Option (c) preserved — no predicates on `Measure`.

### 3.2 Aliases (M9 — attach to `measure.dag`, no per-site re-declarations)

| Alias | Definition | Census fact |
|-------|------------|-------------|
| `ByteSize` | `Measure<Memory, One>` | RAM / capacity bytes |
| `Hertz` | `Measure<Frequency, One>` | per-thread Hz |
| `HardwareThreadCount` | `Measure<Count, One>` | CPU hardware threads (new — names the concept `std.cpu` currently models as bare `Int`) |
| `MoneyAmount<S>` | `Measure<Currency, S>` | price magnitude (Phase 4) |

### 3.3 Constructors / projections (per-alias totals — no generic `measure<Q,S>`)

```dag
fn byte_size(count: Int) -> ByteSize
fn byte_size_count(b: ByteSize) -> Int
fn hertz(count: Int) -> Hertz
fn hertz_count(h: Hertz) -> Int
fn hardware_thread_count(count: Int) -> HardwareThreadCount
fn hardware_thread_count_value(t: HardwareThreadCount) -> Int
```

Projections are **scaffold** until G2 lands (field access through aliases on v2); consumers
use projections in fn bodies until interpreter chases aliases.

---

## 4. Full conversion plan (census — manager-provided)

### 4.1 In scope — kernel / std (this worker; **no `compute_fabric.dag`**)

| File | Fields / data to convert | Notes |
|------|--------------------------|-------|
| `std/measure.dag` | `Measure<Q,S>` body + aliases + fn scaffold | kernel authority |
| `std/memory/sk_hynix.dag` | `stick_capacity_bytes: Int` → `ByteSize`; catalog `capacity_bytes` | data authority |
| `std/memory/types.dag` | `DramModuleCatalogRow.capacity_bytes`, `HostRamSupplyFacts.{nominal,observed}_*` | |
| `std/memory/operator_fleet.dag` | `supply_srv*_observed_ram_bytes`, nominal data rows | wraps fleet RAM |
| `std/cpu/types.dag` | `CpuModelCatalogRow.threads`, `nominal_sustained_per_thread_hz`, `CpuDeploymentFacts.sustained_per_thread_hz` | → `HardwareThreadCount`, `Hertz` |
| `std/placement_supply.dag` | `PlacementSupplyRow.{hardware_threads, clock_hz, ram_bytes}` | v2-safe import graph preserved |

**`placement_supply` import rule:** may import `std.measure` + `std.types` only (no
`float.dag` / `compute_fabric`).

### 4.2 Out of scope — manager coordinates after #4810

| File | Notes |
|------|-------|
| `std/compute_fabric.dag` | **DO NOT TOUCH** in kernel PR — `MemoryFacts.capacity` etc. already typed `ByteSize` but uninhabited; conversion sequences after #4810 |
| `CostEstimate` / Hetzner `*_eur_micros` stopgap | Phase 4 — `MoneyAmount` + currency code; manager lane |

### 4.3 HARD CARVE-OUT — extdeps stay spec-faithful

Per extdeps fidelity invariant: **do not replace raw API integers with `Measure` inside
extdeps.** At most add a **projection** at the std boundary.

| extdeps site | Spec shape | Boundary action |
|--------------|------------|-----------------|
| `extdeps/docker.dag` `TmpfsMount.size_bytes: Int` | Docker tmpfs size int | `byte_size(size_bytes)` at ingest if needed |
| `extdeps/cloud/gcp/secret_manager.dag` `max_secret_size_bytes: Int` | GCP API int | same |
| `gunbc/workflow` `BlobRef.size_bytes` | wire int | same |

---

## 5. Phased landing

| Phase | Deliverable | v3 typecheck | v2 claim-run |
|-------|-------------|--------------|--------------|
| **1** | `measure.dag` value body + aliases + constructors | ✅ expected | ❌ G1–G3 |
| **2** | `cpu/types`, `memory/types`, `sk_hynix`, `operator_fleet` | ✅ | ❌ (use projections in fn bodies if needed) |
| **3** | `placement_supply` typed fields | ✅ | ❌ until G1–G3 |
| **4** | Currency / `MoneyAmount` (director scale decision) | TBD | TBD |
| **5** | `compute_fabric` (manager, post-#4810) | — | — |

**Escalation (recommended with Phase 1):** v2 interpreter lane for G1 (Quantity/Scale as
phantom type args), G2 (alias field chase in infer), G3 (parametric record literals in
`data`/expr). Without this, marks stay honest — "typed in model, v2 execution uses
projections."

---

## 6. Currency open point (Phase 4 — needs director ratification)

`CostEstimate` today: `Float expected_units` + `NonEmptyStr currency`. Hetzner
`*_eur_micros` stopgap is integer micros — aligns with **minor-unit `MoneyAmount<One>` +
ISO currency code** (recommend over `Float`). SI `Scale` on `Currency` is not financially
standard; do not force `Milli` = cents without catalog authority per currency.

---

## 7. Acceptance criteria

1. Single authority per fact — no parallel bare `Int` at converted std sites.
2. `Measure` authority only in `measure.dag` (M2).
3. Option (c) — no `Refined<Measure<…>>` at this layer.
4. extdeps raw ints untouched; projections only at boundary.
5. `compute_fabric.dag` untouched in kernel PR.
6. v3 bootstrap/ratchet green after Phase 1–2.

---

## 8. Test plan

| Check | When |
|-------|------|
| v3 bootstrap loads widened `Measure` | Phase 1 |
| Spike witnesses (non-parametric control) | done — v2 green |
| `placement_supply` v2 claim-run with typed fields | after G1–G3 |
| extdeps files unchanged by diff | every PR |

---

**Awaiting manager/operator confirmation before kernel implementation.**
