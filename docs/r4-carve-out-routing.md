# R4 Carve-Out Routing — items deferred from R3 per Director ratification 2026-05-06

**Authority parent**: R3 Substrate Manager (#1739)
**Ratification**: Director ratified S2 Q2 (option-(b) T-LBP narrowing) + S3 Q-Axis-Enumeration (Phase-1 minimum) + S8 Q-Rounding-Mode (separate-axis, Phase-1 2-axis) at gunbc#828 inbox response 2026-05-06 (zesty-bear-812)
**Authority docs**:
- `docs/briefs/r3-substrate-s2-t-lbp-scope-calibration-canvas.md`
- `docs/briefs/r3-substrate-s3-machine-constraint-carrier-worker.md`
- `docs/briefs/r3-substrate-s8-approximate-field-float-migration-worker.md`
- `docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md`
- `docs/r3-design-schedule-2026-05-06.md`

## Purpose

Per Director option-(b) ratification on S2: T-Lens-Behavioral-Parity
narrows R3 scope to **complexity + cost lenses only**; remaining
T-LBP work + several adjacent substrate items carve to R4 (or
later horizon). This document is the canonical routing record
for the carved items so downstream Mgrs and the R4 program-plan
authors can consume them as a closed set.

This is **not** a worker brief. It is a routing ledger. Items
listed here exit R3 §1.8 ledger scope; their substrate gaps re-enter
the R4 program plan as inputs.

## Carved-out items

### C1 — `parallelism_lens_behaviorally_complete` (was §1.8 row #81)

**R3 status**: carved to R4 per S2 option-(b) ratification.
**Substrate gap**: Stage 2e parallelism walker port from
`src/v3/compiler/src/workflow_parallelism.rs` to `.dag`; rewire
via `lane2_workflow_at` / `std.effects` (idempotency closure
template). Substrate exists; work is port + rewire.
**R4 dispatch shape**: M-sized lane in Substrate-continuation
program. Independent from any other R4 critical-path; can dispatch
parallel.
**Cascade impact in R3**: T-LAS demonstration
`opt_in_iteration_parallelism_via_lens_application_demonstrated`
(originally part of T-LAS gates) carves alongside this lens — the
demo cannot land without parallelism lens COMPLETE.

### C2 — `effect_enumeration_lens_behaviorally_complete` (was §1.8 row #82)

**R3 status**: carved to R4 per S2 option-(b) ratification.
**Substrate gap**:
- 4a: resource-threading migration (substrate exists; consumer migration)
- 4b: ambient metadata removal (cleanup)
- 4c: caller-side effect-set pinning carrier — **NEW substrate
  introduction required** (not landed at R3 horizon)
- 4d: full `OperationEffect` retirement (depends on 4a-4c)
**R4 dispatch shape**: 4c is substrate-fact-introduction (P1
procedure); brief authoring + worker dispatch in R4 Substrate
continuation. 4a-4b-4d sequence after.
**Cascade impact in R3**: none beyond T-LAS demonstration scope
(no T-LAS demo specifically gates on effect_enumeration COMPLETE);
T-LBP "register zero proxy / zero stub" gate (#83) narrows scope
to in-R3 lenses (see C3).

### C3 — `lens_capability_register_zero_proxy_zero_stub` scope narrowing (§1.8 row #83)

**R3 status**: gate **redefined** for R3 horizon. Original closure
predicate "register status updated to ZERO PROXY / ZERO STUB at R3
close" narrows to "ZERO PROXY / ZERO STUB **for in-scope lenses
(complexity + cost)**". Parallelism + effect_enumeration entries in
register remain at PROXY / STUB / PARTIAL with documented R4
carve-out reference.
**R4 dispatch shape**: at R4 close, register reaches full ZERO
PROXY / ZERO STUB across all 4 lenses. In R3, gate #83 closes
under the narrowed scope.

### C4 — Deferred `MachineConstraint<C>` axes per S3 Q-ratification

**R3 status**: S3 Phase-1 lands `MachineWidth<bits>` +
`MachineConstraint<C>` + `AlgebraMachineProduct` only. Per Director
ratification (`feedback_construction_over_ratchets`-aware), additional
axes defer until concrete consumer demand surfaces.
**Carved axes**:
- `RegisterClass<R>` (int / float / vector register classes)
- `EndianMode<E>` (Little / Big)
- `Alignment<bytes>` (Nat-valued phantom)
**R4 dispatch trigger**: per-axis consumer demand surfaces. No
speculative landing in R3 or R4. Each axis lands when an emission
consumer (Grounding) demonstrates need (e.g., a target-language
emission rule cannot produce correct output without
`EndianMode` discrimination).

### C5 — Rounding-mode product-shape extension per S8 Q-ratification

**R3 status**: S8 Phase-1 lands `AlgebraMachineProduct` as 2-axis
(algebra × machine). Per Director ratification, rounding-mode
enters via product-shape extension in a follow-up — NOT R3.
**Substrate shape**: `AlgebraMachineRoundingProduct` (or
equivalent 3-axis interaction-lookup carrier) extends the
`AlgebraMachineProduct` table. Specific shape decision deferred
to follow-up brief.
**R4 dispatch trigger**: when a consumer (Grounding lens or emission
rule) needs rounding-mode-aware emission. Consumer-demand-driven,
not speculative.

### C6 — Aspect-axis (PointKind) follow-on for `EpochMs` and instant-shaped refinements

**R3 status**: 2-axis `Measure<Quantity, Scale>` carrier landing IS
in R3 (Q-Unit-1..5 RATIFIED; outer carrier name finalized as
`Measure<Q, S>` post-Q-Unit-1-Recanvas at gunbc#828
#issuecomment-4385539791 — informal label "Unit/Quantity carrier"
and formal type name `Measure<Q, S>` disambiguated here for
future readers).
S9 Phase-3 dimensional refinements `Duration` / `Seconds` /
`Milliseconds` reframe to outer-Refined / inner-Measure form and
land in R3.

**Carved to R4**: the **Aspect axis** (`PointKind = Magnitude |
Instant | Rate` or equivalent) — adds a third axis distinguishing
duration-shaped from instant-shaped from rate-shaped quantities.
`EpochMs` (instant-shaped) requires Aspect to land correctly; per
Q-Unit-2 RATIFIED 2-axis Phase-1, `EpochMs` defers to R4 alongside
the Aspect-axis introduction. `Frequency` and `DataRate` (Quantity
values currently in 2-axis) may re-fold under Aspect when it
lands; accepted as future cleanup.

**R4 dispatch trigger**: a second instant-shaped or rate-shaped
consumer beyond `EpochMs` surfaces concrete demand for Aspect axis
(per `feedback_construction_over_ratchets` — no speculative
landing). Substrate Mgr authors Aspect-axis carrier brief; emission
consumers (Grounding) wire downstream.

## Cascade messages (cross-Mgr consumption)

### To Verification Mgr (#1740) — T-LAS / T-LSA cross-program impact

- **T-LAS carve-out**: `opt_in_iteration_parallelism_via_lens_application_demonstrated`
  carves with C1 (parallelism lens). Other T-LAS demos
  (complexity-contract-compile-error / CRDT cost basis / memory-peak
  cost basis) **stay in R3** — all are complexity / cost-driven.
- **T-LSA timing-lens-driven**: T-Lens-Self-Application uses
  **timing lens** (T-WAD substrate), not T-LBP scope. T-LSA
  demonstration **stays in R3** under option-(b); not carved.
- **T-WAD timing-lens carrier**: stays in R3 (separate from T-LBP
  scope per Substrate canvas + Director ratification).

### To Grounding Mgr (#1745) — emission consumer scope

- **C4** (deferred MachineConstraint axes): Grounding does NOT
  need to consume `RegisterClass` / `EndianMode` / `Alignment` in
  R3. Grounding emission-rules consume `AlgebraMachineProduct`
  with 2-axis shape (algebra × machine width).
- **C5** (rounding-mode): Grounding does NOT need rounding-mode-
  aware emission in R3. If a specific Grounding row needs it,
  surface as concrete consumer demand for C5 R4 dispatch trigger.
- **C6** (Aspect-axis follow-on): 2-axis `Measure<Q, S>` carrier
  landing is in R3 (Substrate Mgr authoring + Unit/Quantity worker
  brief). Grounding consumes 2-axis dimensional refinements
  (`Duration`, `Milliseconds`, `Seconds`) in R3 emission. R4 C6
  scope is the Aspect-axis introduction for `EpochMs` (instant-
  shaped) + future instant/rate-shaped refinements only.

### To Debt-Paydown Mgr (#1744) — drift items

- C1-C3 (T-LBP scope narrowing) introduces ROADMAP debt-row
  entries: "T-Lens-Behavioral-Parity (parallelism + effect_enumeration)
  carved to R4 per Director ratification 2026-05-06". Debt-Paydown
  reconciliation captures these in `r3_debt_paydown_zero_remaining`
  Pass surface as **explicitly-carved-out**, not as Open drift.
- C4-C6 substrate-axis deferrals also enter the carved-out ledger
  (not drift).

## R4 program plan input

When the R4 program plan is authored, the items above are inputs:

1. C1 — parallelism lens completion (Substrate continuation)
2. C2 — effect_enumeration lens completion + 4c carrier introduction
   (Substrate continuation; Practice 4 / P1 substrate-fact-introduction
   procedure for 4c)
3. C3 — `lens_capability_register_zero_proxy_zero_stub` full closure
   across all 4 lenses
4. C4 — additional MachineConstraint axes per consumer demand
5. C5 — rounding-mode product-shape extension per consumer demand
6. C6 — Aspect-axis (PointKind) follow-on for `EpochMs` and future instant/rate-shaped refinements (2-axis Measure carrier already in R3)

## Provenance

Drafted 2026-05-06 per Director ratification of S2/S3/S8/S9 at
gunbc#828 inbox response 2026-05-06 (zesty-bear-812). Routing ledger
serves R3 closure receipts (carved-out items are NOT R3 drift) and
R4 program-plan input (carved-out items ARE R4 inputs).
