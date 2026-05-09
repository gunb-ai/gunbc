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
| **F-β.1** | C2 | #82 (canvas) | **migration-shape ratification canvas** — Operation field/walker rewire shape per locked design `docs/design-effect-enumeration-resource-threading.md` §3.2 + §6.2; surfaces: which fields of `services.dag::Operation` the lens body reads, walker rewire surface, test-consumer breaking changes | Substrate Mgr standing-authority canvas; Director ratifies surfaced questions |
| **F-β.2** | C2 | #82 (impl) | atomic-migration implementation per F-β.1 ratified shape; 4a/4b/4d bounded migration over existing `services.dag::Operation` carrier (no new substrate type) | post-canvas-ratification worker dispatch |
| **F-γ.1** | (was C3) | #95 demo | opt-in iteration parallelism worked-example demo via `apply_lens(parallelism, fn, Enforce { ... })` | cascade post-F-α (parallelism BEHAVIORALLY COMPLETE) + T-LAS Slice B landing (#91 per-lens LensEnforcement projection) |
| **F-γ.2** | (#83 narrowing dissolution) | #83 register full-scope | register status update — fires for all 4 in-R3 lenses (complexity + cost + parallelism + effect_enum) at R3 close | cascade post-**all 4 lenses BEHAVIORALLY COMPLETE** = F-α (#81 parallelism) + F-β.2 (#82 effect_enum) + #79 (complexity, T-LBP existing) + #80 (cost, T-LBP existing) |

**F-α + F-β.1 parallel-dispatchable** on Task 12 merge (both substrate-ready; F-β.1 is canvas-tier work, not blocked on substrate). F-β.2 cascade-gates on F-β.1 ratification. F-γ.1 cascade-gates on F-α + T-LAS Slice B. F-γ.2 cascade-gates on all 4 lenses BEHAVIORALLY COMPLETE.

**Director sequencing per gunbc#846 #issuecomment-4412475559** (revising prior (a-corrected) at #issuecomment-4412433924): F-β.1 stays as separate migration-shape ratification canvas phase — migration shape decisions (Operation field reads / walker rewire surface / test-consumer breaking changes) are substantive enough to warrant early Director-tier ratification per `feedback_construction_over_ratchets`. Pattern: **substrate-shape canvases for novel substrate; migration-shape canvases for non-trivial ports**. Both surface decisions upfront.

The locked-design discipline holds (no new top-level carrier per §3.2 + §6.2; uses existing `services.dag::Operation`); F-β.1 is canvas-tier ratification of migration shape, not substrate-fact-introduction.

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

### §1.2 F-β.1 — C2 #82 migration-shape ratification canvas

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) — canvas authoring under standing authority.

**Scope (per Director ratification at gunbc#846 #issuecomment-4412475559 — revised from prior (a-corrected) collapse)**: migration-shape ratification canvas (NOT new-substrate-introduction; substrate authority remains the existing `services.dag::Operation` carrier per locked design §3.2 + §6.2). Pattern: **migration-shape canvases for non-trivial ports** — surface decisions upfront per `feedback_construction_over_ratchets`.

Director's sequencing reasoning:
> "Migration shape decisions that warrant early ratification:
> - Which fields of `services.dag::Operation` does the lens body read?
> - Walker rewire surface on Operation
> - Test-consumer breaking changes
> These are smaller than full substrate-shape questions but still worth Director-tier ratification before worker port."

**Locked-design citations** (canvas authoring substrate):
- Design §3.2: "The pinning *substrate carrier* already exists: `src/v3/std/services.dag::Operation`. No new top-level carrier is required."
- Design §6.2: "Operation already exists. ... No new substrate type."

**Migration-shape questions for canvas to surface** (Director ratifies):
- Which fields of `services.dag::Operation` does the new lens body read?
- Walker rewire surface (lens body kind-classification dispatch)
- Test-consumer breaking changes (e.g., effect_enumeration test fixtures that consume the old hand-Rust walker)
- Sequencing of 4a (resource-threading migration) / 4b (ambient metadata removal) / 4c (lens body change) / 4d (full OperationEffect retirement)

**Receipt**:
- Canvas ratified by Director (with migration-shape dispositions on Operation field reads / walker rewire surface / test-consumer changes).
- F-β.2 implementation worker dispatches against locked migration shape.

### §1.3 F-β.2 — C2 #82 atomic-migration implementation

**Owner**: Substrate Mgr worker (post-F-β.1 canvas-ratification dispatch).

**Scope** (per F-β.1 ratified migration shape): atomic migration per design §6.2 (single-PR shippable). Same shape as F-α walker port — port-and-rewire bounded work using existing `services.dag::Operation` carrier (no new substrate type per design §3.2/§6.2).

- **4a — resource-threading migration**: substrate exists; consumer migration only (bounded)
- **4b — ambient metadata removal**: cleanup (bounded)
- **4c — lens body change (kind-classification dispatch)**: lens reads effect set from arrow signature directly + dispatches on `callable_inhabits` lookup per design §2.4(b). Bounded; no new substrate.
- **4d — full `OperationEffect` retirement**: depends on 4a-4c convergence.

**Receipt**:
- Effect-enumeration walker hand-Rust retires (drops from `EXPECTED_HAND_AUTHORED_NON_TEST`).
- `Operation` carrier consumed as substrate-fact authority for caller-side effect-set pinning (per design §3.2).
- Gate #82 status: DECLARED → CONSUMER_LANDED → PASSING.
- ~3 entries dissolve from SG-0 census.

**Receipt**:
- Effect-enumeration walker hand-Rust retires (drops from SG-0).
- Gate #82 status: DECLARED → CONSUMER_LANDED → PASSING.
- ~3 entries dissolve from SG-0 census.

### §1.4 F-γ — split into F-γ.1 (#95 demo) + F-γ.2 (#83 register)

**Split rationale (per codex BLOCKING on PR #2364 sha `14e4d8ff6` line 22)**: prior single-phase F-γ collapsed two gates with different prerequisite sets — #95 cascades on F-α + T-LAS Slice B (parallelism + apply_lens only), while #83 register cascades on ALL 4 lenses BEHAVIORALLY COMPLETE. Each gate gets one canonical close predicate per INVARIANTS P2.

#### §1.4.1 F-γ.1 — C3 #95 demo

**Owner**: Verification Mgr (wise-bear-525 / gunbc#2075).

**Scope**: opt-in cross-iteration parallelism via lens application — worked example via `apply_lens(parallelism, fn, Enforce { ... })`.

**Cascade prerequisites**:
- F-α completion (parallelism BEHAVIORALLY COMPLETE)
- T-LAS Slice B landing (per-lens LensEnforcement projection #91 + violation routing)

**Receipt**: #95 status DECLARED → CONSUMER_LANDED → PASSING (worked-example demo).

#### §1.4.2 F-γ.2 — #83 register full-scope

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) for register state update; cross-program with Verification.

**Scope**: dissolution of prior "scope narrowing" framing (formerly C3 in carve doc); register fires for ALL 4 in-R3 lenses (complexity + cost + parallelism + effect_enum) at R3 close.

**Cascade prerequisites** (all 4 lenses BEHAVIORALLY COMPLETE):
- F-α completion (#81 parallelism)
- F-β completion (#82 effect_enum)
- #79 (complexity lens BEHAVIORALLY COMPLETE — existing T-LBP scope)
- #80 (cost lens BEHAVIORALLY COMPLETE — existing T-LBP scope)

**Receipt**: #83 status DECLARED → CONSUMER_LANDED → PASSING (register reports ZERO PROXY / ZERO STUB for all 4 in-R3 lenses).

---

## §2. Cross-Mgr coordination

| Phase | Substrate Mgr | Verification Mgr | PB Mgr |
|---|---|---|---|
| F-α | author worker brief; dispatch | (none) | (none) |
| F-β.1 | author canvas (migration-shape ratification) | (none) | (none) |
| F-β.2 | atomic-migration worker (port + rewire; same shape as F-α) | (none) | (none) |
| F-γ.1 | (none) | (#95 demo worker) | (none) |
| F-γ.2 | (#83 register status update — small) | (none) | (none) |

T-LAS Slice B landing is a separate T-Lens-Application-Surface lane work item (not a Cluster F sub-phase); its sequencing is tracked in T-LAS lane-Mgr scope (Substrate Mgr per `r3-structure.md` §"Lane structure"). F-γ.1 cascade-gates on T-LAS Slice B landing as an external dependency.

---

## §3. Velocity-to-zero contribution

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) §3.3 + Director's velocity update at #issuecomment-4412330468 + #issuecomment-4412380947:

**Cluster F bulk-dissolution events (carve-promotion contribution)**:
- F-α: ~3 entries dissolve (workflow_parallelism.rs + supporting files)
- F-β.2: ~3 entries dissolve (effect-enum walker hand-Rust; post-F-β.1 canvas ratification)
- F-γ.1: ~2 entries dissolve (#95 demo wiring; varies by implementation shape)
- F-γ.2: 0 entries dissolve (#83 is a register-status update, not a hand-Rust retirement)

Plus existing Cluster F scope (#5/#6/#7 LP-retirement + #71 self-host trampoline): ~14 entries dissolve.

**Cluster F total**: ~22 entries dissolve via Cluster F bulk events. Combined with Cluster M's ~80-90 test-side dissolution, accounts for ~100+ of the 150 SG-0 entries (full PB-0 closure).

---

## §4. Sequencing within R3 window (8-12 weeks)

Per Director's bulk-event sequencing in #issuecomment-4412008376 §C, refined by carve-promotion at #issuecomment-4412380947:

| Bulk event | Cycle window | Cluster F phase |
|---|---|---|
| Cluster M Phase 1 (#85/#86) | weeks 1-4 | (parallel with F-α + F-β.1) |
| Cluster M Phase 2 (#87) | weeks 2-5 | (parallel) |
| Cluster M Phase 3 (#84 bulk-port) | weeks 3-7 | (parallel with F-γ tail) |
| **F-α (C1 walker port)** | **weeks 1-3** | parallel with Cluster M Phase 1 |
| **F-β.1 (C2 migration-shape canvas)** | **weeks 1-2** | parallel with F-α (canvas-tier work; substrate-ready) |
| **F-β.2 (C2 atomic-migration implementation)** | **weeks 2-4** | post-F-β.1 ratification |
| **F-γ.1 (#95 demo)** | **weeks 4-6** | post-F-α + T-LAS Slice B |
| **F-γ.2 (#83 register full-scope)** | **weeks 4-7** | post-all-4-lenses-BEHAVIORALLY-COMPLETE (F-α + F-β.2 + #79 + #80) |
| Tail: per-file/small-class | weeks 6-8 | various |

All phases bounded; staffing-not-a-concern directive applies. Fits 8-12 week R3 window.

---

## §5. Operator-tier spawn-authority queue update

Per Director's #issuecomment-4412392036:
> "Spawn-authority for new workers (multiple): consolidated queue at PM #846; operator-tier ratification batched."

Cluster F adds to the spawn queue (alongside Cluster M Phase 1 #85/#86 canvases):
- **F-α worker** (C1 walker port; Substrate Mgr brief)
- **F-β.1 canvas-author** (Substrate Mgr standing authority — no operator spawn needed; mgr-tier canvas-drafting)
- **F-β.2 atomic-migration worker** (post-F-β.1 ratification; Substrate Mgr brief; same shape as F-α port + rewire)
- **F-γ.1 #95 demo worker** (Verification Mgr brief; post-F-α + Slice B)
- **F-γ.2 #83 register full-scope** (Substrate Mgr small-scope status update; post-all-4-lenses-BEHAVIORALLY-COMPLETE — no operator spawn needed)

Operator authorizes spawns; Substrate/Verification Mgrs dispatch under standing authority.

---

## §6. Dispatch readiness checklist

After Task 12 carve-promotion amendment PR ratifies + merges:
- [ ] Substrate Mgr dispatched on F-α (C1 walker port worker brief)
- [ ] Substrate Mgr dispatched on F-β.1 (C2 migration-shape ratification canvas under standing authority)
- [ ] F-β.1 canvas surfaces migration-shape questions → Director ratifies → F-β.2 worker brief + dispatch
- [ ] F-α completion + T-LAS Slice B landing → Verification Mgr dispatches F-γ.1 #95 worker
- [ ] F-α (#81) + F-β.2 (#82) BEHAVIORALLY COMPLETE + existing #79/#80 → F-γ.2 #83 register full-scope status update
- [ ] §1.8 ledger Status updated for #81/#82/#83/#95 as each gate transitions DECLARED → CONSUMER_LANDED → PASSING
- [ ] `r4-carve-out-routing.md` C1/C2/C3 entries amended to dissolved-status (in this Task 12 PR)

---

**End of plan.**
