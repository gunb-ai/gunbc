# Unit scale+quantity anti-pattern — remediation sequencing plan

**Status:** ACTIVE (sequencing artifact). GATE 0 executes in this lane; GATE 1+ park
behind value-Measure ratification.
**Frozen:** 2026-06-15.
**Owner:** fierce-carp-10 (audit, for bold-crane-680) → remediation child fierce-tern-760.
**Carrier authority:** `dsl/std/measure.dag` (snappy-stag-903). Every carrier touch routes
through snappy-stag; nothing re-mints.

> **What this doc is.** A *sequencing* artifact: the substrate gates, their ownership, and
> the order the remediation lands in. It is **not** a parallel ledger of per-field facts.
> The authoritative home for each field's status is the inline `🟡`/`🟢` mark in its own
> file plus the held SPICE PRs (#4894 / #4895). The per-field census in
> [§4](#4-frozen-census-snapshot-appendix) is a **frozen snapshot that points at those
> homes** — it is not maintained here.

---

## 1. The linchpin reframe

The anti-pattern: leaf types bake a **metric scale** + a **physical quantity** into one flat
`Int` (or `Float`) field — e.g. `microvolts: Int`, `*_eur_micros: Int`, `clock_hz: Int`.
The canonical fix is the dimensional carrier `Measure<Q, S>` in `dsl/std/measure.dag`:
phantom `<Q>`/`<S>` tag the quantity and scale, `count: Nat` holds the magnitude.

**GATE 0 is a TYPE-LEVEL annotation, not a value-carrying runtime path.** The
`Measure<Q, S>` / `ByteSize` / `Hertz` type migration **typechecks** in v4 today — the
`count: Nat` field (`measure.dag:178`) and the constructor fns compile — but the **value
semantics are not available yet**: the v2 interpreter cannot read `.count` through the
carrier (the G3 gap, e.g. the `money_micros_count` multi-hop read at `measure.dag:232`), and
value-typed inhabitants pend value-Measure. So a GATE-0 migration is a dimensional type
annotation that compiles; it is **not** a `money_micros()` runtime projection that
round-trips today. The word "phantom" in the `measure.dag` header refers to the `<Q>`/`<S>`
dimensional *tags*; the broader point here is that the *magnitude* does not yet flow through
the carrier at runtime on v2.

What is *not* yet built, beyond runtime value flow, is the **signed/continuous**
generalization (`Measure<Q, S, M>` over the numeric tower) needed for signal samples, deltas,
and fractional readings.

So the census below is real, but the **sequencing is gated**: a migration only *lands* as a
real value path when its magnitude domain *and* the v2 read are supported. Most of the census
sits behind value-Measure follow-ons. GATE 0 lands only where the change is purely type-level
and **no consumer reads the value** (see §3).

**Boundary-projection shape (the GATE-0 template).** extdeps stay spec-faithful — raw API
ints keep their declared `Int` type *inside* extdeps. The wrap onto a `Measure` alias happens
at the **std consumer boundary** via the constructor fn — the structural shape
`std.compute_fabric` uses for money (`money_micros(cax41_catalog.hourly_eur_micros)` at
`compute_fabric.dag:597-603`). This is the *type-level* template to follow; it compiles, but
the wrapped magnitude does not round-trip on v2 until GATE 3. Do not lift raw ints into std;
do not change extdeps field types.

**PINNED-vs-MAIN skew.** `money_micros()` and the value constructors are gunbc-**main**-only;
ctrl's pinned tree (`third_party/gunbc @ 81829fc2`) lags main and carries a barer `Measure`.
This skew matters only to smart-hawk-763's eventual ctrl pin-bump, not to gunbc remediation —
and it is why ctrl's `placement.dag` currently compiles against `Int`.

---

## 2. Gate sequence

| Gate | What unblocks it | Owner | Scope unblocked |
|------|------------------|-------|-----------------|
| **GATE 0** | **nothing — type-level annotation, typechecks today** | this lane | non-negative integer-at-scale leaves where the change is purely type-level **and no consumer reads the value**: annotate onto `ByteSize`/`Hertz`/`HardwareThreadCount`/`MoneyMicros` at the std boundary (money_micros structural shape). The wrapped magnitude does not round-trip on v2 until GATE 3. |
| **GATE 1** | value-Measure: signed-real magnitude `Measure<Q, S, M>{count: M}` over the numeric tower (Nat→Int→Rational→Real/Float) | snappy-stag-903 (proposing up via zesty-otter-413; co-design gentle-lynx) | SIGNED or CONTINUOUS magnitudes: signal samples, differentials/deltas, fractional readings — incl. SPICE signal fields and any fractional device rating. |
| **GATE 2** | Quantity-enum extension (Q-Unit-5 P1: each new value lands *with* a consuming rule) | snappy-stag-903 | `ElectricPotential` / `ElectricCurrent` / `Resistance` / `Capacitance` / `Inductance` / `Power` / `Temperature` (physics-quantity names, `Scale=One` base). Unblocks SPICE device *ratings* + `tdp_watts`. |
| **GATE 3** | v2 multi-hop alias `.count` read (snappy-stag #4885 / G3) | snappy-stag-903 | end-to-end v2 **evaluation** of multi-hop carrier values (e.g. `MoneyMicros.count`). Construction/projection at boundaries already resolves single-hop — GATE 0 does not wait on this. |
| **GATE 4** | `Refined<Measure<…>, predicate>` substrate lane (option (c) deferred at gunbc#828) | snappy-stag-903 decides whether to escalate | bounded-range fields (`lifetime_seconds` `range(min:1,max:3600)`) **without dropping the bound** — see interim rule below. |
| **GATE 5** | std-root `types.dag` brand-Int dissolution | **ESCALATE** — high-bar standalone PR (load-bearing `types.dag` + consumers) | `EpochMs` / `Duration` / `Milliseconds` / `Seconds`. Do **not** fold into extdeps cleanup. |

**Interim Refined rule (snappy-stag-decreed).** When migrating a bounded-range field, do
**not** drop the bound. If the bound cannot be preserved on the `Measure` carrier today,
**HOLD** the field (leave it as-is) rather than silently lose it — dropping it is an M12
regression. snappy-stag owns whether `Refined<Measure>` becomes a substrate lane (GATE 4).

**TERMINAL (leave as-is):** `src/v3/std/substrate.dag` `PerfBaselineMeasurement`
(`median_ns` / `p99_delta_ns`) — delta encoding is design-deliberate (enforces `p99 >= median`).

---

## 3. GATE 0 — executed in this lane

| Site | Change | Status |
|------|--------|--------|
| `dsl/std/placement_supply.dag:33` `cpu_capacity_hz_row` | returned flat `Int` (thread-count × per-thread-Hz, dimension dropped at the std boundary) → returns `Hertz` via `hertz()`. **Valid precisely because it is type-level and the fn has zero consumers** — nothing reads the value, so the v2 `.count` gap (GATE 3) is not exercised. | **done (this PR)** |

**GATE-0 follow-ups (not in this PR):**
- `dsl/std/cpu/types.dag` `CpuModelCatalogRow.cores: Int` — `Count`-dimension integer, but
  `HardwareThreadCount` names *threads*, not cores. Needs a `CoreCount = Measure<Count, One>`
  alias minted on `measure.dag` (coordinate with snappy-stag — carrier touch). Integral-Nat,
  so unblocked once the alias exists.
- `dsl/std/cpu/types.dag:25` `tdp_watts: Int` — `Power` quantity → **GATE 2** (Power not in
  the `Quantity` enum yet).

Much of the std-internal type-level migration already landed before this lane: `byte_size()`
/ `hertz()` / `hardware_thread_count()` / `money_micros()` already annotate data literals
across `std.memory`, `std.cpu/ampere`, and `std.compute_fabric` (these typecheck in v4; the
magnitudes do not yet flow through the carrier at runtime on v2). GATE 0 here closes the
remaining std-boundary dimension-drop at the type level.

---

## 4. Frozen census (snapshot appendix)

> **FROZEN 2026-06-15 SNAPSHOT — NOT MAINTAINED.** The authoritative home for each field's
> status is the inline `🟡`/`🟢` mark in its own file and the held SPICE PRs #4894 / #4895.
> This appendix is a pointer index for sequencing only; do not treat it as a live ledger or
> update it in place. Repos swept: gunbc (merged) + ctrl (merged); SPICE PRs pending from
> zesty-swift-79.

### gunbc (merged tree)

- **Tier A — std-root, un-acknowledged (high-bar):** `dsl/std/types.dag` `EpochMs` /
  `Duration` / `Milliseconds` / `Seconds` (brand-Int temporal) → GATE 5 (escalate).
  `dsl/std/placement_supply.dag:33` `cpu_capacity_hz_row` → **fixed (GATE 0, this PR)**.
- **Tier B — self-acknowledged scaffolds:** `src/v3/std/timing_lens.dag` `Nanoseconds`
  (🟡 cites `Measure<Time,S>` deferral); `dsl/extdeps/cloud/hetzner.dag:22-25`
  `memory_bytes`/`disk_bytes` (ByteSize) + `hourly_eur_micros`/`monthly_cap_eur_micros`
  (MoneyMicros — currency half already wrapped at std boundary,
  `compute_fabric.dag:597-603`). extdeps stay spec-faithful; wrap at the consumer boundary
  **when** a std consumer reads them (the hetzner physical facts are not yet wired into std —
  `compute_fabric.dag:582-590` `processors/memory/storage: []`).
- **Tier C — naked extdeps/domain fields (spec-faithful boundary):** time/byte/rate fields in
  `cloud.dag`, `gcp/iam.dag` (bounded → GATE 4), `gcp/secret_manager.dag`, `auth/patterns.dag`
  (bounded → GATE 4), `browser/browser.dag`, `docker/docker.dag`, `workflow/types.dag`,
  `std/memory/types.dag` (DataRate), `std/cpu/types.dag` `tdp_watts` (Power → GATE 2).
- **Tier D — TERMINAL:** `src/v3/std/substrate.dag` `PerfBaselineMeasurement`. Leave as-is.
- **src/v4:** clean (zero).

### ctrl (merged tree) — OUT OF THIS LANE

Manager routed ctrl remediation to **smart-hawk-763** (ctrl-homed). Symptom recorded for
sequencing only: `plans/lib/placement.dag` re-declares flat `Int` for fields with gunbc
`ByteSize`/`Hertz` in scope (strips the dimension). `workflows/branch_review.dag:122`
`timeout_seconds:Int` (inconsistent with gunbc `Milliseconds` use at L114). Good exemplar
(do not touch): `plans/draft/agent_mapreduce.dag` consumes gunbc Measure-typed `WorkDemand`.

### SPICE PRs (zesty-swift-79) — held pending operator-ratified shape

`#4895` (`src/v4/std/timeseries_signal.dag`) and `#4894` (`dsl/std/circuit.dag` + v4 mirror)
introduce a **second axis** of the anti-pattern: raw `Float` for a dimensioned quantity.
Two field classes:
- **device RATINGS** (positive; e.g. `ohms`, `farads`, `henries`, `dt_seconds`) → GATE 2
  (needs the electrical Quantity values) **and** GATE 1 where the rating is fractional
  (`Measure.count` is `Nat`; `4.7 ohms` = `4700` at `Milli` only if expressed as integer-at-
  scale, else waits on the Rational step of value-Measure).
- **SIGNAL SAMPLES** (signed/continuous; `magnitude`, `volts`, `amperes`, `charge_coulombs`,
  `flux_webers`) → **GATE 1** (signed-real `Measure<Q, S, M>`). Do not wrap onto Nat aliases.

Authoritative status lives in the PRs themselves; this is a pointer.

---

## 5. Ownership & guardrails

- All carrier changes route through **snappy-stag-903** (`measure.dag` authority); never
  re-mint or fork the carrier.
- ctrl placement remediation → **smart-hawk-763** (different repo/owner).
- New Quantity-enum values land **with a consumer** (Q-Unit-5 P1), coordinated with snappy-stag.
- A review hard-blocker (proud-wren-735) should gate so no *new* flat-scalar unit leaf is
  introduced while remediation is pending.
- The remaining gated remediation re-engages via a value-Measure-gated backlog item when
  GATE 1 lands; this lane stands down after GATE 0 + this doc.
