# R3 R4-Carve Substrate-Readiness Audit (C1/C2/C3) — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier substrate-readiness audit per Director ratification at gunbc#846 #issuecomment-4412330468 (2026-05-09; Director RATIFIED PM (α) carve-promotion-IN-R3 recommendation per operator's 2026-05-09 framing "R3 close = 0 hand-Rust including tests AND stage0; bootstrap is data + self-generated; no need to edit stage0 ever").
**Parent docs**:
- [`docs/r4-carve-out-routing.md`](../r4-carve-out-routing.md) — current carve enumeration (C1/C2/C3 will dissolve per this audit's recommendations)
- [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.5 + §1.8 (gates #81/#82/#95)
- [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) — original drift finding that surfaced workflow_parallelism.rs

---

## §0. Executive summary

Per Director's three outcomes:

| Carve | Gate | Substrate state | Recommendation |
|---|---|---|---|
| **C1** parallelism lens behaviorally complete | **#81** | **substrate-ready** — `effects.dag` provides EffectShape/OperationEffect/CompositionVerdict; workflow_parallelism.rs imports map cleanly to substrate; port-and-rewire bounded (M-sized lane) | **Full carve-promotion to R3** — fold into Cluster F |
| **C2** effect_enumeration lens behaviorally complete | **#82** | **MIXED** — 4a/4b/4d bounded (consumer migration + cleanup); 4c (caller-side effect-set pinning carrier) is **NEW P1 substrate-fact-introduction** not landed at HEAD | **Director disposition needed**: (a) full carve-promotion (4c lands in R3 as new substrate intro per "staffing not concern") OR (b) (γ) `.dag`-stub-form for #82 declarations (NYI bodies until 4c lands; substantive R4 work) |
| **C3** opt-in iteration parallelism via lens application demonstrated | **#95** | **substrate-ready conditional on C1** — `lens_application.dag` exists; cascade-gated on parallelism lens BEHAVIORALLY COMPLETE | **Full carve-promotion to R3** — lands when C1 lands |

**Net**: 2 of 3 carves are substrate-ready for full promotion (C1 + C3); 1 has a specific substrate-cliff (C2 4c).

**Decision-state scope clarification (added 2026-05-09 per openai-pro REQUEST_CHANGES)**:

- **(α) carve-promotion-IN-R3 thesis is RATIFIED** at gunbc#846 c#4412330468 — broad framework: R4 carves C1/C2/C3 dissolve; gates #81/#82/#95 are R3-load-bearing. This is settled.
- **C2 #82 sub-disposition (a) vs (γ-stub) is OPEN** — a finer-grain decision *within* the ratified (α) framework. Whether to author 4c P1 substrate-fact-introduction in R3 (path (a)) or land #82 as `.dag`-stub-form with NYI bodies (path (γ)) is the open Director disposition. Both paths satisfy operator's strict-zero hand-Rust framing; they differ on whether #82's behavioral completion happens in R3 (path (a)) or is deferred to R4 with R3-stub-form retirement of the hand-Rust file (path (γ)).

The two scopes do not contradict; (α) is ratified at thesis level, (a) vs (γ) is the open sub-decision under (α). The "Recommendation" column for C2 lists both (a) and (γ) as options for Director ratification at the sub-disposition level.

---

## §1. C1 — parallelism lens behaviorally complete (#81)

### §1.1 Substrate state

- `src/v3/std/effects.dag` — exists. Carries:
  - `EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`
  - `OperationEffect` (collapsed from `DerivedOpEffect` per PR #521)
  - `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: ElementRef<OperationEffect> }`
  - `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict`
  - `derive_effect_shape(method, path)` — HTTP method + path → EffectShape
- `src/v3/std/workflows.dag` — exists.

### §1.2 Hand-Rust producer

`src/v3/compiler/src/workflow_parallelism.rs` (sole file, ~250 lines).

**Imports from substrate Rust mirror**:
```rust
use crate::dag::{
    CompositionVerdict, Dag, EffectShape, ElementRef, IdempotentShape, NodeId, NonSingletonList,
    OperationEffect, ParallelismUnsupportedDetail, ParallelismUnsupportedKind, WorkflowEffect,
    WorkflowParallelismReport,
};
```

All imported types are **substrate-resident** (via `effects.dag` + workflows.dag). The walker is a fold over these types projecting `WorkflowParallelismReport`.

### §1.3 Port shape

Per carve doc: "Stage 2e parallelism walker port from `src/v3/compiler/src/workflow_parallelism.rs` to `.dag`; rewire via `lane2_workflow_at` / `std.effects` (idempotency closure template). Substrate exists; work is port + rewire."

- **Bounded scope** — single file, single fold, substrate types pre-existing
- **M-sized lane** per carve doc estimate
- **No new substrate-fact-introduction needed**

### §1.4 Recommendation

**Full carve-promotion to R3.** Fold into Cluster F (T-LP-Retirement) framework alongside existing LP-retirement program. Gate #81 reclassifies from R4-carved → R3-load-bearing.

Bulk-dissolution event: ~3 entries collapse from `EXPECTED_HAND_AUTHORED_NON_TEST` (workflow_parallelism.rs + 1-2 supporting files if any).

---

## §2. C2 — effect_enumeration lens behaviorally complete (#82)

### §2.1 Substrate state

- `src/v3/std/effects.dag` + `dsl/std/effects.dag` — both exist with substrate carriers above.
- **No 4c carrier at HEAD**: grep for `EffectSet`, `EffectPin`, `caller-side effect-set` in `effects.dag` returns no matches. Per carve doc: "4c: caller-side effect-set pinning carrier — **NEW substrate introduction required** (not landed at R3 horizon)."

### §2.2 Hand-Rust producer

- `src/v3/compiler/src/lens_effect_enumeration_generated.rs` — **GENERATED** (not hand-Rust; doesn't count toward SG-0).
- The hand-Rust walker analog is distributed across `lens_apply.rs`, `dag.rs` (via OperationEffect handling), etc. Per carve doc 4-task split:
  - **4a**: resource-threading migration (substrate exists; consumer migration only — bounded)
  - **4b**: ambient metadata removal (cleanup — bounded)
  - **4c**: caller-side effect-set pinning carrier — **NEW substrate intro** (NOT bounded; P1 substrate-fact-introduction procedure)
  - **4d**: full `OperationEffect` retirement (depends on 4a-4c)

### §2.3 Substrate-cliff specifics

4c is the cliff. **Two distinct gating layers** (corrected 2026-05-09 per openai-pro REQUEST_CHANGES on PR #2363 sha `c3a4b110` — prior framing folded both into one "P1 procedure" label):

**Layer 1 — `INVARIANTS.md` P1 substrate-fact-introduction modeling checks (per [`INVARIANTS.md`](../../INVARIANTS.md):94-129)**:

1. **DAG-ancestor check** ([`INVARIANTS.md`](../../INVARIANTS.md):100-108): does an ancestor type already exist? Caller-side effect-set pinning — attaches via inhabitance to existing `EffectShape` / `OperationEffect` parents, or stands as new substrate primitive? Not run at HEAD.
2. **Coproduct-vs-coordinate check** ([`INVARIANTS.md`](../../INVARIANTS.md):109-119): if proposing `EffectSet = A | B | C` shape, are these alternatives (one-at-a-time) or coordinates (all-at-once record)?
3. **Primitive-vs-lens-extensible check** ([`INVARIANTS.md`](../../INVARIANTS.md):120-128): substrate primitive (sibling to existing primitives) or lens-extensible label?

These three modeling checks **must surface in the canvas authoring before carrier shape is ratified**. Without running them, carve-promotion authorizes a substrate addition without proving it's the right substrate fact.

**Layer 2 — Dispatch / process gates (separate from P1 modeling checks)**:

1. Confirmed bridge consumer (downstream consumer demand surfaces)
2. Carrier shape ratification (post-Layer-1 modeling checks)
3. Worker brief authoring
4. Substrate-introduction PR

Layer 1 is the P1 modeling discipline; Layer 2 is dispatch/process discipline. Both are required.

This is **substantive substrate authoring**, not port-and-rewire. Operator's "staffing not concern" directive permits this scope, but it expands R3 substrate intro work — and the canvas must run the P1 Layer 1 checks before recommending a carrier shape.

### §2.4 Recommendation — Director disposition needed

Two paths:

- **(a) Full carve-promotion** — include 4c P1 substrate-fact-introduction in R3 scope. New substrate canvas at `docs/briefs/r3-substrate-effect-set-pinning-canvas-2026-05-09.md`; Substrate Mgr dispatches under standing authority. R3 scope expands by ~M-sized substrate work (analog to Cluster M Phase 1 #85/#86 canvases). Per operator framing + "staffing not concern" — this is the canonical strict-zero answer.

- **(b) (γ) `.dag`-stub-form interim** — gate #82 lands as `.dag` declaration with `NotYetImplemented` body for the effect_enumeration lens producer; 4c carrier authoring deferred to R4. Hand-Rust file (if any) retires (declarations exist in .dag); SG-0 census drops correctly. Behavioral completion of #82 stays R4 work.

**PM recommendation: (a)** — aligns with operator framing "no need to edit stage0 ever" + "0 hand-Rust including stage0" + "staffing not concern." (γ) interim drops a behavioral gap into the 93-load-bearing set; (a) closes it strictly.

---

## §3. C3 — opt-in iteration parallelism via lens application (#95)

### §3.1 Substrate state

- `src/v3/std/lens_application.dag` — exists. Per file body lines 30/88/94: parallelism is mentioned as one of the lenses the apply_lens carrier supports.
- **Cascade dependency**: per carve doc, "Pass requires parallelism lens BEHAVIORALLY COMPLETE (`docs/design-lens-application-surface.md` §4.4 / §7)." So #95 lands when C1 #81 lands.

### §3.2 Substrate readiness

Substrate-ready conditional on C1. lens_application.dag is the consumer surface; once parallelism lens behavior is in `.dag` form (post-C1 port), #95's demonstration follows naturally.

### §3.3 Recommendation

**Full carve-promotion to R3.** Cascade-gated on C1; lands when C1 lands. Gate #95 reclassifies from R4-carved → R3-load-bearing.

---

## §4. Cluster F sequencing folder for #81/#82/#95

Per Director's structural framing, all three carve gates are **lens-producer-retirement** work. Fold into Cluster F (T-LP-Retirement):

| Sub-gate | Scope | Owner | Phase |
|---|---|---|---|
| **#81** parallelism lens | port `workflow_parallelism.rs` → `.dag` (M-sized lane; substrate-ready) | Substrate Mgr (warm-wolf-698) | Cluster F sub-phase α |
| **#82** effect_enum lens | per Director disposition: (a) full + 4c canvas OR (b) γ stub | Substrate Mgr (warm-wolf-698) | Cluster F sub-phase β (depends on disposition) |
| **#95** opt-in iteration parallelism demo | lens_application demonstration (cascade post-#81) | Verification Mgr (wise-bear-525) | Cluster F sub-phase γ (post-#81) |

Cluster F existing scope (#5/#6/#7 LP-retirement + #71 self-host trampoline) absorbs these as additional sub-gates per "lens-producer-retirement" structural framing.

---

## §5. Open questions for Director ratification

1. **C2 #82 disposition**: (a) full carve-promotion with 4c canvas authoring in R3 OR (b) (γ) `.dag`-stub-form interim with 4c R4-carved? PM recommends (a) per operator strict-zero framing.

2. **Cluster F sequencing within itself**: do #81 + #82 + #95 sub-phases run sequentially (α → β → γ) or are #81 + #82 parallel-dispatchable (independent substrate; same Mgr)? PM recommends parallel for #81+#82 (no mutual substrate dep), serial for #95 (cascade-gated on #81).

3. **R4 carve doc dissolution**: amend `docs/r4-carve-out-routing.md` to remove C1/C2/C3? Per Director's ratification language ("R4 carves dissolve") — yes; PM authors amendment in carve-promotion PR.

---

## §6. Cycle-aggregate update

Director's velocity-to-zero update per #846 #issuecomment-4412330468:
- Original audit identified 4-6 bulk-dissolution events
- Carve-promotion adds: C1 walker port (+1 bulk event), C2 walker port (+1 if (a)), C3 lens application demo (+1)
- **Updated estimate: 5-9 bulk events** for full PB-0 + carve-promotion R3 close

Each bounded substrate / cluster work; staffing-not-a-concern directive applies.

---

## §7. Out-of-scope follow-ups

Per Director's #846 #issuecomment-4412330468: **UNACCOUNTED entries in non-test census** need named retirement gates. PM follow-up grep authoring (separate audit doc) — for each unaccounted entry, surface the gap so Director can assign it a Cluster (F / K / M / V2-Retirement / new). Director: "If no Cluster fits, that's a structural gap requiring fresh substrate authoring."

This audit's scope is C1/C2/C3 only; UNACCOUNTED grep is a separate follow-up artifact (Task 13).

---

**End of audit.**
