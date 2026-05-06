---
status: draft (Mgr-tier canvas; ENGAGE-NOW per Director disposition 2026-05-06)
authority parent: R3 Substrate Manager (#1739) + Verification Manager (#1740) cross-program
ratification: Director needs Substrate canvas detail before engaging Q-Lens-Behavioral-Parity-R3-Closeability (E3 RED)
roadmap row: docs/r3-program-plan.md §10 Q-LBP-R3-Closeability + §1.8 ledger rows #79-#83 (T-LBP), #61 (Class 2), #88-#95 (T-LAS), #54 (timing-lens), #57-#59 (T-LSA)
authority docs:
  - docs/r3-structure.md §"Summary" T-LBP lane (L-XL sized; cross-program Substrate+Verification)
  - docs/r3-structure.md §"Acceptance — `.dag` gates" T-LBP gate IDs
  - docs/r3-structure.md §"Lane structure" (lane row + 4 sub-slices)
  - docs/v3-lens-capability-register.md (PROXY/STUB/PARTIAL classifications)
  - docs/r3-program-plan.md §10 Q-LBP-R3-Closeability
  - docs/r3-design-schedule-2026-05-06.md §1 S2
gates:
  - complexity_lens_behaviorally_complete (#79)
  - cost_lens_behaviorally_complete (#80)
  - parallelism_lens_behaviorally_complete (#81)
  - effect_enumeration_lens_behaviorally_complete (#82)
  - lens_capability_register_zero_proxy_zero_stub (#83)
---

# R3 Substrate S2 — T-Lens-Behavioral-Parity scope-calibration canvas

## Purpose

Director needs Substrate-side canvas detail before engaging
Q-Lens-Behavioral-Parity-R3-Closeability (currently E3 RED — no
executable close path identified). Per `r3-structure.md` §"Lane structure" T-LBP
has 4 sub-slices. This canvas surfaces, for each lens, the specific
substrate / runtime / consumer blockers that gate
"PROXY/STUB/PARTIAL → BEHAVIORALLY COMPLETE", and names option
(a) / (b) / (c) shapes consistent with Substrate Mgr E3 RED:

- **(a) accept**: T-LBP scope unchanged; Director ratifies finite
  close-path within R3 horizon for all 4 lenses.
- **(b) reframe**: narrow R3 T-LBP scope to 1-2 lenses; remaining
  lenses carve to R4 with substrate-gap routing.
- **(c) carve to R4**: T-LBP fully out of R3; cascading lanes
  (T-LAS / T-WAD / T-LSA / Class 2) re-routed or carved.

This canvas is **Mgr-tier scope-calibration input**, not a worker
brief. Output → Director ratifies (a/b/c) shape → cascading dispatch
work follows.

## Per-lens blocker matrix

Per `docs/v3-lens-capability-register.md` audit (Substrate Mgr surface
of register status; worker re-verifies at HEAD before ratification).

### Lens 1 — `complexity` (currently PROXY)

Sub-slice scope (per `r3-structure.md` §"Lane structure"): symbolic CostExpr full
algebra (Sum/Mul/Log/Const) consumed by lens; work/span dimension
split; asymptotic classification; cementing test against frozen
v2-oracle snapshot.

| Sub-slice | Blocker | Substrate dependency | Status |
|---|---|---|---|
| 1a — symbolic CostExpr algebra consumed | Lens reads `per_call_descent_evidence` side-table at full coverage | T-E-P-Producer-Broadening Phase 1 | foundational; in flight (S10) |
| 1b — work/span split | Lens emits `Dimension<Work>` + `Dimension<Span>` separately rather than scalar cost | substrate `Dimension<C>` projection — landed | landed |
| 1c — asymptotic classification | Lens computes asymptotic class from symbolic algebra | depends on 1a + classification fold | gated on 1a |
| 1d — cementing test (frozen v2-oracle) | Frozen snapshot captured pre-v2-retirement; cementing-test consumes frozen receipt | T-Tests-As-Data-Completeness consumer pattern; frozen snapshot infrastructure | T-V2-Retirement coordination |

**Close-path option-(a) feasibility**: feasible IF T-E-P Phase 1
lands + cementing-test pattern lands. No substrate-extension required.
Critical-path: longest among 4 lenses but not blocked by missing
substrate.

### Lens 2 — `cost` (currently PROXY)

Sub-slice scope: same producer foundation as complexity; `SizeVar`
value semantics; `Dimension<SymbolicCost>` wiring; cementing test.

| Sub-slice | Blocker | Substrate dependency | Status |
|---|---|---|---|
| 2a — `SizeVar` value semantics | `SizeVar` substrate landed; lens evaluates against runtime values | landed (per `design-cost-lens-sizevar-dimension-wiring.md`) | landed |
| 2b — `Dimension<SymbolicCost>` wiring | Lens output projects `Dimension<SymbolicCost>` | landed (Dimension projection) | landed |
| 2c — producer consumption | same as 1a — `per_call_descent_evidence` full coverage | T-E-P Phase 1 | gated on T-E-P |
| 2d — cementing test | same as 1d | same | T-V2-Retirement coordination |

**Close-path option-(a) feasibility**: feasible; mirrors complexity
critical-path. Both lenses share the producer dependency; closing
T-E-P Phase 1 unblocks both simultaneously.

### Lens 3 — `parallelism` (currently STUB)

Sub-slice scope: Stage 2e parallelism walk port from
`src/v3/compiler/src/workflow_parallelism.rs` to `.dag`; rewire via
`lane2_workflow_at` / `std.effects` (idempotency closure template).

| Sub-slice | Blocker | Substrate dependency | Status |
|---|---|---|---|
| 3a — Stage 2e walk ported to `.dag` | Workflow-parallelism walker needs `.dag` representation | substrate: `lane2_workflow_at` query + `std.effects` idempotency template | partial (effects substrate landed; walker port pending) |
| 3b — rewire via `lane2_workflow_at` | Lens consumes the new query | depends on 3a | gated on 3a |
| 3c — cementing test | frozen v2-oracle parallelism trace | same as 1d | T-V2-Retirement coordination |

**Close-path option-(a) feasibility**: feasible; substrate exists,
work is port + rewire. Independent from T-E-P critical-path. Sized
M (smaller than complexity/cost).

### Lens 4 — `effect_enumeration` (currently PARTIAL)

Sub-slice scope: resource-threading migration; ambient metadata
removal; caller-side effect-set pinning; full `OperationEffect`
retirement.

| Sub-slice | Blocker | Substrate dependency | Status |
|---|---|---|---|
| 4a — resource-threading migration | Effects route through resource handles (not ambient metadata) | substrate: existing resource carrier (Read/Write/Exclusive); migration target | substrate landed |
| 4b — ambient metadata removal | `OperationEffect` carrier no longer needed for caller-side enumeration | substrate cleanup; coordinate with T-V2-Retirement | gated on 4a |
| 4c — caller-side effect-set pinning | Lens reads pinned effect-set from caller, not callee ambient | substrate: caller-side pinning carrier | NOT LANDED — substrate gap |
| 4d — full `OperationEffect` retirement | Tier-2 retirement of legacy carrier | depends on 4a-4c | gated on 4a-4c |

**Close-path option-(a) feasibility**: BLOCKED on 4c — caller-side
effect-set pinning carrier is not landed. This is a **substrate
extension required for option-(a) feasibility**. If 4c is not
authored within R3 horizon, lens 4 cannot reach BEHAVIORALLY
COMPLETE.

## Option (a/b/c) shape recommendations

### Option (a) — accept full T-LBP scope

**Feasibility per blocker matrix**: feasible for lenses 1, 2, 3.
**BLOCKED for lens 4** on substrate gap 4c (caller-side effect-set
pinning carrier).

**Required for (a)**: Substrate Mgr authors brief for 4c carrier;
worker dispatched in R3. Adds approximately 1 worker-slot
equivalent to R3 critical path (lens 4 is currently PARTIAL, not
STUB; partial-to-COMPLETE work plus 4c carrier authoring).

**Cost**: ~M-L worker effort; 4c carrier introduction is substrate-
fact-introduction event (P1 procedure required).

### Option (b) — reframe T-LBP to 1-2 lenses for R3

**Recommended subset**: complexity + cost (lenses 1 + 2). Both share
the T-E-P producer dependency; closing them simultaneously is the
critical-path fastest path. Both have frozen-v2-oracle cementing
test pattern available.

**Carve-to-R4**: parallelism (lens 3) + effect_enumeration (lens 4).
Class 2 gap-test (S1 candidate) does NOT require parallelism or
effect_enumeration COMPLETE — re-uses complexity/cost lens
infrastructure for the function-valued data path. Class 2 closure
remains feasible under (b).

**Cost**: ~M worker effort reduction; parallelism + effect_enumeration
substrate-gaps (4c) carry to R4 with documented routing.

**Cascade impact**: T-LAS (lens application surface) gates on
"lenses COMPLETE". Under (b), T-LAS demonstrations narrow to
complexity-contract-compile-error + CRDT cost basis (both consume
complexity / cost). memory-peak cost basis is cost-lens-driven (still
in scope). opt-in cross-iteration parallelism demonstration (T-LAS
gate `opt_in_iteration_parallelism_via_lens_application_demonstrated`)
becomes problematic — needs parallelism lens. Carve to R4 alongside
parallelism lens.

T-Lens-Self-Application demonstration uses **timing** lens (T-WAD's
new lens shape, not in T-LBP scope). T-LSA still feasible under (b).

### Option (c) — T-LBP fully out of R3

**Cascade impact**: massive. T-LAS, T-WAD, T-LSA all carry to R4 or
later. R3 critical-path collapses. Recursive-flex thesis claim
(`r3-structure.md` §"Summary" recursive-flex thesis claim) un-cashes for R3.

**Recommendation against (c)** unless Director judges that R3 close
horizon is fundamentally incompatible with any T-LBP work.

## Ratification surface

- **Q1**: Is option (a) acceptable given the 4c substrate gap? If
  yes, Substrate Mgr authors 4c carrier brief immediately + dispatch
  to worker pool.
- **Q2**: Is option (b) acceptable scope-narrow? If yes, Substrate
  Mgr documents R4 carve-out routing for lenses 3 + 4; cascade
  recipients (T-LAS, T-WAD, T-LSA Mgrs) consume the narrowed scope.
- **Q3**: If neither (a) nor (b), surface Director-requested option
  shape; this canvas iterates.

## STOP-AND-ESCALATE

- **Lens capability register drift**: this canvas is anchored to
  register status as of 2026-05-06. If register is updated mid-
  canvas, blocker matrix may shift. Worker re-verifies register
  at HEAD before ratification proceeds.
- **T-E-P-Producer-Broadening reveals additional producer surface
  required for lens 1/2 BEHAVIORALLY COMPLETE**: the canvas
  assumes T-E-P Phase 1 is sufficient producer surface. If
  Phase 1 lands and complexity/cost lenses still cannot reach
  COMPLETE, re-canvas; option-(a) feasibility for lenses 1/2
  shifts.
- **Class 2 gap-test (S1) requires lens-behavioral-parity COMPLETE
  for representative**: S1 canvas argues otherwise (function-valued
  data path is evaluator-side, not lens-behavior-dependent). If
  Director rules S1 narrowing infeasible, T-LBP becomes load-bearing
  for Class 2 — option (b) carve-out narrows further (cannot
  exclude complexity/cost without breaking S1).

## Authority audit receipt

1. **Substrate exists?** Per-lens substrate inventoried in blocker
   matrix; 4c is the only confirmed substrate gap. Lenses 1-3
   substrate is landed; work is consumer-wiring + cementing tests.
2. **Existing brief?** None for T-LBP scope-calibration. T-LBP
   itself was authored 2026-05-02 per `r3-structure.md` expansion;
   this canvas is the scope-calibration response to E3 RED.
3. **Design-doc match?** `v3-lens-capability-register.md` is the
   anchor. Worker re-reads at dispatch.
4. **Citations live?** Verified at HEAD 2026-05-06: `r3-structure.md` §"Summary" + §"Acceptance — `.dag` gates" + §"Lane structure"
   and design schedule §1 S2.
5. **Carrier dissolves the bridge?** N/A — canvas is scope-calibration
   not carrier landing. The "bridge" is the YELLOW chain rule
   violation in Q-LBP-R3-Closeability + Class 2 cascade. (a/b/c)
   recommendation provides Director with shape options to dissolve.

## Provenance

Drafted 2026-05-06 per R3 design schedule §1 S2 (PR #1810). Director
ratifies (a/b/c) shape; cascade dispatch to T-LAS / T-WAD / T-LSA
Mgrs follows. Coordinate with Verification Mgr (#1740) for
cross-program V3 sub-slice ownership.
