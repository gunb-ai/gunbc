# R3 Cluster F (Lens-Producer-Retirement) Sequencing Plan — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier sequencing plan per Director ratification at gunbc#846 #issuecomment-4412380947 (2026-05-09; Director RATIFIED (a) full carve-promotion including 4c) + greenlit Task 12 amendment PR at #issuecomment-4412392036.
**Parent docs**:
- [`docs/audit/r3-r4-carve-substrate-readiness-2026-05-09.md`](r3-r4-carve-substrate-readiness-2026-05-09.md) — substrate-readiness audit findings (PR #2363; sha `32afcd329`)
- [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 gates #81/#82/#83/#95
- [`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) §3.2 + §6.2 — locked-design authority for C2 atomic migration
- Operator framing 2026-05-09 ("0 hand-Rust including tests AND stage0; bootstrap is data + self-generated") + Director ratification 2026-05-09 c#4412330468 (carve-promotion-IN-R3 thesis)

---

## §0. Executive summary

Cluster F (T-LP-Retirement) framework absorbs C1/C2/C3 carve-promotion per Director (a) ratification + audit substrate-readiness findings. Three carve gates promote to R3-load-bearing; #83 scope-narrowing dissolves alongside (since underlying lenses no longer carved).

| Phase | Carve | Gate | Work | Substrate-readiness |
|---|---|---|---|---|
| **F-α** | C1 | #81 parallelism | walker port from `workflow_parallelism.rs` → `.dag`; rewire via `lane2_workflow_at` / `std.effects` | substrate-ready (M-sized port) |
| **F-β** | C2 | #82 effect_enum | **atomic migration** using existing `services.dag::Operation` carrier per locked design `docs/design-effect-enumeration-resource-threading.md` §3.2 + §6.2; lens body change (kind-classification dispatch) + 4a/4b/4d bounded migration | substrate-ready (no canvas needed; same shape as F-α port) |
| **F-γ** | C3 | #95 demo + #83 register | opt-in iteration parallelism demo + register status update for all 4 lenses | cascade post-F-α + T-LAS Slice B |

F-α + F-β **parallel-dispatchable** (both substrate-ready, no canvas-tier blocker). F-γ cascade-gates on F-α completion + T-LAS Slice B landing.

Per Director (a-corrected) ratification at gunbc#846 #issuecomment-4412433924 (refining the initial (a) at #issuecomment-4412380947 against locked design doc): F-β collapses from prior 2-phase (canvas + implementation) to single-phase atomic migration. No new substrate canvas required. Director cited PM round-1 audit fix at sha `530376d50` as structurally correct; (a-as-stated) would have authored parallel 4c carrier alongside existing `Operation` — `feedback_parallel_representation_debt` violation.

---

## §1. Phase scopes

### §1.1 F-α — C1 #81 parallelism walker port

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068).

**Scope**: port `src/v3/compiler/src/workflow_parallelism.rs` to `.dag`. Per audit §1: substrate exists (`effects.dag` provides EffectShape/OperationEffect/CompositionVerdict; workflow_parallelism.rs imports map cleanly to substrate). Port + rewire bounded (M-sized lane per carve doc estimate).

**Receipt**:
- `src/v3/compiler/src/workflow_parallelism.rs` retires (drops from `EXPECTED_HAND_AUTHORED_NON_TEST`).
- `.dag` parallelism walker lands in `src/v3/std/` (or `src/v3/lenses/`).
- Gate #81 status: DECLARED → CONSUMER_LANDED → PASSING.
- ~3 entries dissolve from SG-0 census.

### §1.2 F-β — C2 #82 effect_enumeration atomic migration

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068).

**Scope (per Director (a-corrected) ratification at gunbc#846 #issuecomment-4412433924)**: atomic migration using existing `services.dag::Operation` carrier per locked design [`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) §3.2 + §6.2.

Director cited:
> "PR #2363 round-1 audit fix at sha `530376d50` is structurally correct. (a-as-stated) would author parallel 4c carrier alongside existing `Operation` — direct violation of `feedback_parallel_representation_debt`. Locked design doc is single-authority per P2."

**Substrate guidance** (locked-design authority):
- Design §3.2: "The pinning *substrate carrier* already exists: `src/v3/std/services.dag::Operation`. No new top-level carrier is required."
- Design §6.2: "Operation already exists. ... No new substrate type."

**Migration shape** (per design §6.2 atomic-migration shape):
- **4a — resource-threading migration**: substrate exists; consumer migration only (bounded)
- **4b — ambient metadata removal**: cleanup (bounded)
- **4c — lens body change (kind-classification dispatch)**: lens reads effect set from arrow signature directly + dispatches on `callable_inhabits` lookup per design §2.4(b). Bounded; no new substrate.
- **4d — full `OperationEffect` retirement**: depends on 4a-4c convergence.

Single-PR shippable per design §6.2 "Why atomic is feasible (not aspirational)." Same shape as F-α walker port — port-and-rewire bounded work.

**Receipt**:
- Effect-enumeration walker hand-Rust retires (drops from `EXPECTED_HAND_AUTHORED_NON_TEST`).
- `Operation` carrier consumed as substrate-fact authority for caller-side effect-set pinning (per design §3.2).
- Gate #82 status: DECLARED → CONSUMER_LANDED → PASSING.
- ~3 entries dissolve from SG-0 census.

**Receipt**:
- Effect-enumeration walker hand-Rust retires (drops from SG-0).
- Gate #82 status: DECLARED → CONSUMER_LANDED → PASSING.
- ~3 entries dissolve from SG-0 census.

### §1.3 F-γ — C3 #95 demo + #83 register status

**Owner**: Verification Mgr (wise-bear-525 / gunbc#2075) for #95; cross-program with Substrate for #83 register update.

**Scope (corrected per audit §3)**:
- **#95 demo**: opt-in cross-iteration parallelism via lens application — worked example via `apply_lens(parallelism, fn, Enforce { ... })`. Cascade-gates on:
  - F-α completion (parallelism BEHAVIORALLY COMPLETE)
  - T-LAS Slice B landing (per-lens LensEnforcement projection #91 + violation routing)
- **#83 register**: dissolves "scope narrowing" framing (formerly C3 in carve doc); register fires for ALL 4 in-R3 lenses (complexity + cost + parallelism + effect_enum) at R3 close.

**Receipt**:
- #95 status: DECLARED → CONSUMER_LANDED → PASSING (worked-example demo).
- #83 status: DECLARED → CONSUMER_LANDED → PASSING (register zero-proxy/zero-stub for all 4 lenses).

---

## §2. Cross-Mgr coordination

| Phase | Substrate Mgr | Verification Mgr | PB Mgr |
|---|---|---|---|
| F-α | author worker brief; dispatch | (none) | (none) |
| F-β | atomic-migration worker (port + rewire; same shape as F-α) | (none) | (none) |
| F-γ | (#83 register) | (#95 demo) | (none) |

T-LAS Slice B landing is a separate T-Lens-Application-Surface lane work item (not a Cluster F sub-phase); its sequencing is tracked in T-LAS lane-Mgr scope (Substrate Mgr per `r3-structure.md` §"Lane structure"). F-γ cascade-gates on T-LAS Slice B landing as an external dependency.

---

## §3. Velocity-to-zero contribution

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) §3.3 + Director's velocity update at #issuecomment-4412330468 + #issuecomment-4412380947:

**Cluster F bulk-dissolution events (carve-promotion contribution)**:
- F-α: ~3 entries dissolve (workflow_parallelism.rs + supporting files)
- F-β: ~3 entries dissolve (effect-enum walker hand-Rust)
- F-γ: ~2 entries dissolve (#95 demo wiring; varies by implementation shape)

Plus existing Cluster F scope (#5/#6/#7 LP-retirement + #71 self-host trampoline): ~14 entries dissolve.

**Cluster F total**: ~22 entries dissolve via Cluster F bulk events. Combined with Cluster M's ~80-90 test-side dissolution, accounts for ~100+ of the 150 SG-0 entries (full PB-0 closure).

---

## §4. Sequencing within R3 window (8-12 weeks)

Per Director's bulk-event sequencing in #issuecomment-4412008376 §C, refined by carve-promotion at #issuecomment-4412380947:

| Bulk event | Cycle window | Cluster F phase |
|---|---|---|
| Cluster M Phase 1 (#85/#86) | weeks 1-4 | (parallel with F-α + F-β) |
| Cluster M Phase 2 (#87) | weeks 2-5 | (parallel) |
| Cluster M Phase 3 (#84 bulk-port) | weeks 3-7 | (parallel with F-γ tail) |
| **F-α (C1 walker port)** | **weeks 1-3** | parallel with Cluster M Phase 1 |
| **F-β (C2 atomic migration)** | **weeks 1-3** | parallel with F-α (both substrate-ready) |
| **F-γ (#95 demo + #83 register)** | **weeks 4-7** | post-F-α + T-LAS Slice B |
| Tail: per-file/small-class | weeks 6-8 | various |

All phases bounded; staffing-not-a-concern directive applies. Fits 8-12 week R3 window.

---

## §5. Operator-tier spawn-authority queue update

Per Director's #issuecomment-4412392036:
> "Spawn-authority for new workers (multiple): consolidated queue at PM #846; operator-tier ratification batched."

Cluster F adds to the spawn queue (alongside Cluster M Phase 1 #85/#86 canvases):
- **F-α worker** (C1 walker port; Substrate Mgr brief)
- **F-β atomic-migration worker** (Substrate Mgr brief; same shape as F-α port + rewire; substrate-ready per design §6.2)
- **F-γ #95 demo worker** (Verification Mgr brief; post-F-α + Slice B)

Operator authorizes spawns; Substrate/Verification Mgrs dispatch under standing authority.

---

## §6. Dispatch readiness checklist

After Task 12 carve-promotion amendment PR ratifies + merges:
- [ ] Substrate Mgr dispatched on F-α (C1 walker port worker brief)
- [ ] Substrate Mgr dispatched on F-β (C2 atomic-migration worker — port + rewire; same shape as F-α; substrate-ready per design §6.2; no canvas needed)
- [ ] F-α completion + T-LAS Slice B landing → Verification Mgr dispatches F-γ #95 worker
- [ ] §1.8 ledger Status updated for #81/#82/#83/#95 as each gate transitions DECLARED → CONSUMER_LANDED → PASSING
- [ ] `r4-carve-out-routing.md` C1/C2/C3 entries removed (in this Task 12 PR)

---

**End of plan.**
