# Measure value-typing — design (scope / dual-representation dissolution)

**Status:** Investigation complete (2026-06-13). Design targets `count: Nat`.
PR #4812 kernel commit (`count: Int`) must be revised before merge — operator rejected
Int-now/Nat-later. Implementation blocked on manager dispatch.
**Work item:** `node://adhoc-4523b7d2-967`
**Session:** zesty-otter-413 · **Manager:** snappy-stag-903 (compute fabric)
**Authority:** `dsl/std/measure.dag` (Q-Unit-1..5 + option (c) ratified at gunbc#828)

---

## 0. Feasibility findings (updated 2026-06-13 — operator override)

### 0.A Measure `count: Nat` (design target — NOT Int)

Per operator: **`Measure<Q, S> { count: Nat }`** is the only acceptable shape.
`Nanoseconds { count: Nat }` in `v3.std.timing_lens` is the substrate precedent.
Int-literal inhabitants in `Nat` fields type-check on v2 (spike-verified).

### 0.B Nat-in-v2 — CORRECTED: import chain is already v2-green

**Prior diagnosis was wrong.** Spike 2026-06-13:

| Module | v2 `gunbc run --source-root dsl` |
|--------|----------------------------------|
| `magnitude.dag` | ✅ resolves (1 source) |
| `algebra.dag` | ✅ resolves (2 sources) |
| `nat.dag` | ✅ resolves (4 sources) |
| `count: Nat` record field | ✅ type-checks (6 sources) |
| `Box { count: 42 }` with `count: Nat` | ✅ type-checks |

**The `nat → algebra → magnitude` chain does NOT need an import-graph fix** for
`placement_supply` to depend on `std.nat`. The tracked deferral under compute-fabric
manager should be reframed: **Nat value-typing for Measure is not blocked by the Nat
import chain.**

**Remaining Nat-adjacent blockers** (separate from Nat import):

| ID | Blocker | Layer |
|----|---------|-------|
| G1 | `Measure<Memory, One>` — Quantity/Scale variant labels unresolved as type params | v2 infer |
| G2 | `ByteSize.count` — alias field access does not chase to `Measure` record | v2 infer |
| G3 | Parametric record literals in `data` rows | v2 parse/infer |

G1–G3 are **Measure-parametric** gaps, not Nat-chain gaps. Constructors/projections
remain required until G2 lands.

**Minimal path (Nat):** no import-chain work. Sequence: (1) revise PR to `count: Nat`;
(2) land kernel + census with `import std.nat { Nat }`; (3) dispatch G1–G3 v2 lane for
claim-run on typed `placement_supply`.

**Effort (Nat):** kernel revision ~1 PR (swap Int→Nat, add `std.nat` import) — **small**.
G1–G3 v2 lane — **medium** (~3–5 days interpreter/infer; see §0.D).

### 0.C Float-in-v2 — scoped breakage + fix path

`compute_fabric.dag` v2 load fails on transitive `float.dag` → `integer.dag`:

```
dsl/std/integer.dag:48:39: error: expected type expression   // MachineWidth<8>
dsl/std/float.dag:37:42: error: expected type expression     // MachineWidth<32>
```

**Root causes (two, ordered):**

| # | Failure | Fix class | Effort |
|---|---------|-----------|--------|
| F1 | `Compose<…> = Phantom` — `Phantom` unresolved in `machine_constraints.dag:112` | **Modeling:** land terminal `Phantom` opaque in `dsl/std/` (sibling to `Product`/`Coproduct` in `constructors.dag`) | **~2h** |
| F2 | `MachineWidth<8>` — v2 `parse_type_expr` rejects `LitInt` in type-arg position (only `ShIdent` accepted after `<`) | **v2 parser:** extend `finish_type_expr_from_name` / type-arg collection to accept literal-Nat indices per R3 gate #60 | **~1–2 days** + regression tests |

**Not blocking float v2 load in current spike:** `v3.std.approximate_field` resolves when
present in module index (7 sources loaded); primary v2 failure is F2 via `integer.dag`.

**Minimal path (Float):** F1 then F2 in one v2 substrate PR → `integer.dag` +
`float.dag` + `compute_fabric.dag` parse/type-check on v2. No `ApproximateField`
migration needed for v2 entry.

**Effort (Float):** **small–medium** (2–3 days total). F2 is bounded parser work, not a
full interpreter rewrite. **Escalate** only if F2 reveals wider literal-type-arg semantics
gaps across the grammar.

### 0.D Measure on v2 (unchanged — interpreter-gated for claim-run)

G1–G3 still block v2 `claim-run` on typed `placement_supply` even after Nat correction.
v3 bootstrap typecheck path is unaffected.

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
