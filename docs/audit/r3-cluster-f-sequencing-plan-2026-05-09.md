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
| **F-β.1** | C2 | #82 (canvas) | Substrate Mgr authors `r3-substrate-effect-set-pinning-canvas-2026-05-09.md` per Director (a) ratification c#4412380947; canvas surfaces 4c shape question (P1 substrate-fact-introduction OR atomic migration over Operation carrier per design §6.2) | shape-question canvas; Director ratifies disposition |
| **F-β.2** | C2 | #82 (impl) | implementation per F-β.1 ratified shape (atomic migration OR new substrate addition); 4a/4b/4d bounded migration | post-canvas-ratification |
| **F-γ** | C3 | #95 demo + #83 register | opt-in iteration parallelism demo + register status update for all 4 lenses | cascade post-F-α + T-LAS Slice B |

F-α + F-β.1 parallel-dispatchable. F-β.2 cascade-gates on F-β.1 ratification. F-γ cascade-gates on F-α completion + T-LAS Slice B landing.

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

### §1.2 F-β.1 — C2 #82 substrate canvas (4c shape)

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068).

**Scope (per Director ratification at gunbc#846 c#4412380947)**: Substrate Mgr authors `r3-substrate-effect-set-pinning-canvas-2026-05-09.md` (or analogous path under Substrate Mgr discretion) under standing authority. Canvas surfaces 4c shape questions; Director ratifies.

Per Director's (a) ratification framing: "C2 #82 effect_enumeration lens — full carve-promotion including 4c new P1 substrate. Substrate Mgr authors `r3-substrate-effect-set-pinning-canvas-2026-05-09.md` under standing authority; Director ratifies surfaced shape questions; worker dispatched against locked shapes."

**Substrate-shape questions for canvas to surface**:

1. **4c shape**: is this a new top-level substrate-fact-introduction (P1 procedure: DAG-ancestor / coproduct-vs-coordinate / primitive-vs-lens-extensible per `INVARIANTS.md`:94-129), or atomic migration over existing Operation carrier?
2. **Locked-design citation**: [`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) §3.2 + §6.2 says: "The pinning *substrate carrier* already exists at `services.dag::Operation`. No new top-level carrier required." Canvas should reconcile this design authority against Director's "4c new P1 substrate" framing. Likely outcome (PM read): canvas confirms atomic migration sufficient per design §6.2; alternatively surfaces case for additional substrate fact if locked design is incomplete or stale.
3. **Sequencing of 4a (resource-threading migration) / 4b (ambient metadata removal) / 4c (caller-side pinning migration or new carrier) / 4d (full OperationEffect retirement)**.

**Receipt**:
- Canvas ratified by Director (with shape-question dispositions).
- Sequencing for 4a/4b/4c/4d locked.
- F-β.2 implementation worker dispatches against locked shapes.

### §1.3 F-β.2 — C2 #82 implementation

**Owner**: Substrate Mgr worker (post-canvas-ratification dispatch).

**Scope** (post-F-β.1 canvas ratification): implementation per locked-shape disposition. Likely shape per design §6.2 atomic-migration framing (PM reading from locked design):
- 4a: resource-threading migration (substrate exists; consumer migration only)
- 4b: ambient metadata removal (cleanup)
- 4c: lens body change (kind-classification dispatch — read effect set from arrow signature directly + dispatch on callable_inhabits) OR new substrate-fact-introduction (per F-β.1 canvas ratification)
- 4d: full OperationEffect retirement (depends on 4a-4c)

Final 4c shape locked at F-β.1 canvas ratification. F-β.2 worker dispatches against ratified shape (atomic migration over existing Operation carrier OR new substrate addition, depending on canvas outcome).

**Receipt**:
- Effect-enumeration walker hand-Rust retires (drops from SG-0).
- Gate #82 status: DECLARED → CONSUMER_LANDED → PASSING.
- ~3 entries dissolve from SG-0 census.

### §1.4 F-γ — C3 #95 demo + #83 register status

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
| F-β.1 | author canvas | (consumer wiring on canvas-ratification) | (none) |
| F-β.2 | implementation worker | (none) | (none) |
| F-γ | (#83 register) | (#95 demo) | (none) |

T-LAS Slice B landing is a separate T-Lens-Application-Surface lane work item (not a Cluster F sub-phase); its sequencing is tracked in T-LAS lane-Mgr scope (Substrate Mgr per `r3-structure.md` §"Lane structure"). F-γ cascade-gates on T-LAS Slice B landing as an external dependency.

---

## §3. Velocity-to-zero contribution

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) §3.3 + Director's velocity update at #issuecomment-4412330468 + #issuecomment-4412380947:

**Cluster F bulk-dissolution events (carve-promotion contribution)**:
- F-α: ~3 entries dissolve (workflow_parallelism.rs + supporting files)
- F-β.2: ~3 entries dissolve (effect-enum walker hand-Rust)
- F-γ: ~2 entries dissolve (#95 demo wiring; varies by implementation shape)

Plus existing Cluster F scope (#5/#6/#7 LP-retirement + #71 self-host trampoline): ~14 entries dissolve.

**Cluster F total**: ~22 entries dissolve via Cluster F bulk events. Combined with Cluster M's ~80-90 test-side dissolution, accounts for ~100+ of the 150 SG-0 entries (full PB-0 closure).

---

## §4. Sequencing within R3 window (8-12 weeks)

Per Director's bulk-event sequencing in #issuecomment-4412008376 §C, refined by carve-promotion at #issuecomment-4412380947:

| Bulk event | Cycle window | Cluster F phase |
|---|---|---|
| Cluster M Phase 1 (#85/#86) | weeks 1-4 | (parallel with F-α + F-β.1) |
| Cluster M Phase 2 (#87) | weeks 2-5 | (parallel) |
| Cluster M Phase 3 (#84 bulk-port) | weeks 3-7 | (parallel with F-β.2) |
| **F-α (C1 walker port)** | **weeks 1-3** | parallel with Cluster M Phase 1 |
| **F-β.1 (C2 canvas)** | **weeks 1-2** | parallel with F-α |
| **F-β.2 (C2 implementation)** | **weeks 2-4** | post-F-β.1 ratification |
| **F-γ (#95 demo + #83 register)** | **weeks 4-7** | post-F-α + T-LAS Slice B |
| Tail: per-file/small-class | weeks 6-8 | various |

All phases bounded; staffing-not-a-concern directive applies. Fits 8-12 week R3 window.

---

## §5. Operator-tier spawn-authority queue update

Per Director's #issuecomment-4412392036:
> "Spawn-authority for new workers (multiple): consolidated queue at PM #846; operator-tier ratification batched."

Cluster F adds to the spawn queue (alongside Cluster M Phase 1 #85/#86 canvases):
- **F-α worker** (C1 walker port; Substrate Mgr brief)
- **F-β.1 canvas-author** (Substrate Mgr standing authority — no operator spawn needed; mgr-tier canvas-drafting per Director Ask 2 disposition for Cluster M)
- **F-β.2 implementation worker** (post-F-β.1 ratification)
- **F-γ #95 demo worker** (Verification Mgr brief; post-F-α + Slice B)

Operator authorizes spawns; Substrate/Verification Mgrs dispatch under standing authority.

---

## §6. Dispatch readiness checklist

After Task 12 carve-promotion amendment PR ratifies + merges:
- [ ] Substrate Mgr dispatched on F-α (C1 walker port worker brief)
- [ ] Substrate Mgr dispatched on F-β.1 (C2 migration-shape canvas authoring under standing authority)
- [ ] F-β.1 canvas surfaces shape questions → Director ratifies → F-β.2 worker brief + dispatch
- [ ] F-α completion + T-LAS Slice B landing → Verification Mgr dispatches F-γ #95 worker
- [ ] §1.8 ledger Status updated for #81/#82/#83/#95 as each gate transitions DECLARED → CONSUMER_LANDED → PASSING
- [ ] `r4-carve-out-routing.md` C1/C2/C3 entries removed (in this Task 12 PR)

---

**End of plan.**
