# R3 R4-Carve Substrate-Readiness Audit (C1/C2/C3) — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier substrate-readiness audit per Director ratification at gunbc#846 #issuecomment-4412330468 (2026-05-09; Director RATIFIED PM (α) carve-promotion-IN-R3 recommendation per operator's 2026-05-09 framing "R3 close = 0 hand-Rust including tests AND stage0; bootstrap is data + self-generated; no need to edit stage0 ever").
**Parent docs (live state on main)**:
- [`docs/r4-carve-out-routing.md`](../r4-carve-out-routing.md) — current carve enumeration (C1/C2/C3 amend to dissolved-status via concurrent PR #2364 Task 12 amendment)
- [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.5 + §1.8 (gates #81/#82/#95) — live ledger
- [`docs/r3-structure.md`](../r3-structure.md) — live lane authority (T-Lens-Application-Surface lane gates #88-#95)
- [`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) — locked design authority for C2 atomic-migration shape (§3.2 + §6.2)
- [`docs/design-lens-application-surface.md`](../design-lens-application-surface.md) — locked design authority for T-LAS Slice A/B carrier shape (§10 step 2 enforcement-mode violation routing)

**In-flight authorities** (cited inline; not on main at audit-authoring time):
- `docs/audit/r3-pb0-velocity-walk-2026-05-09.md` — original drift finding that surfaced `workflow_parallelism.rs` as the parallelism producer (concurrent PR #2358; routing context only — substrate state of `effects.dag` + `workflow_parallelism.rs` is independently verifiable on main)

---

## §0. Executive summary

Per Director's three outcomes:

| Carve | Gate | Substrate state | Recommendation |
|---|---|---|---|
| **C1** parallelism lens behaviorally complete | **#81** | **substrate-ready** — `effects.dag` provides EffectShape/OperationEffect/CompositionVerdict; workflow_parallelism.rs imports map cleanly to substrate; port-and-rewire bounded (M-sized lane) | **Full carve-promotion to R3** — fold into Cluster F |
| **C2** effect_enumeration lens behaviorally complete | **#82** | **substrate-ready** (corrected 2026-05-09 post-codex BLOCKING) — `services.dag::Operation` carrier already exists per locked design [`design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) §3.2 + §6.2. **No new substrate type required.** Atomic-migration shape feasible per design §6.2. | **Full carve-promotion to R3** — fold into Cluster F (atomic migration; same shape as C1) |
| **C3** opt-in iteration parallelism via lens application demonstrated | **#95** | **substrate-ready conditional on T-LAS Slice B + C1** (corrected 2026-05-09 post-codex BLOCKING) — Slice A landed (#88-#90 PR #2145); Slice B (#91 per-lens LensEnforcement projection + violation routing) not yet landed. #95 is a worked-example demonstration gate, not a separate carve | **Full carve-promotion to R3** — cascade-gates on Slice B + C1 |

**Net (corrected 2026-05-09 post-codex BLOCKING)**: **all 3 carves are substrate-ready for full carve-promotion to R3.** No substrate-cliff. Earlier framing of "C2 4c is a substrate-cliff requiring NEW P1 substrate-fact-introduction" was based on stale carve-doc claim; the locked design at `design-effect-enumeration-resource-threading.md` §3.2 + §6.2 is the more recent authority and says the carrier already exists at `services.dag::Operation`.

**Decision-state status**:
- **(α) carve-promotion-IN-R3 thesis is RATIFIED** at gunbc#846 c#4412330468 — R4 carves C1/C2/C3 dissolve; gates #81/#82/#95 are R3-load-bearing.
- **No open sub-dispositions**: prior "C2 (a) vs (γ-stub)" disposition is **MOOT** — there is no 4c canvas to author since the Operation carrier already exists. C2 simply promotes alongside C1 (port-and-rewire / atomic migration shape).
- **C3 substrate prerequisites**: T-LAS Slice B landing (#91 per-lens LensEnforcement projection + violation routing) + parallelism lens BEHAVIORALLY COMPLETE (C1 #81). These are tracked in their respective lanes; not blockers for the carve-promotion ratification, just sequencing prerequisites for the gate firing.

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

**MAJOR REVISION 2026-05-09 per codex BLOCKING on PR #2363 sha `c3a4b110`**: prior audit relied on grep names (`EffectSet`, `EffectPin`) instead of the locked design authority at [`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md). Codex correct: the design doc §3.2 + §6.2 explicitly says **the carrier already exists**. C2 4c is NOT a substrate-cliff requiring new substrate-fact-introduction.

### §2.1 Locked-design authority (replaces prior grep-based assessment)

[`docs/design-effect-enumeration-resource-threading.md`](../design-effect-enumeration-resource-threading.md) §3.2 — "The pinning surface — `Operation` declarations":
> "The pinning *substrate carrier* already exists: `src/v3/std/services.dag::Operation`."
> "Resource pinning IS the `callable: CallableRef` field plus the threaded signature on the referenced declaration. **No new top-level carrier is required**; the thread-through-signature rule of §2 plus the existing `Operation.callable: CallableRef` provides the pinning surface."

§6.2 — "Why atomic is feasible (not aspirational)":
> "**`Operation` already exists.** `src/v3/std/services.dag:122` declares the carrier with `callable: CallableRef + inputs: Map + endpoint: RestEndpointBinding` — exactly what §3.2 needs. **No new substrate type.**"

§6.2 also: "**`Arrow.body` E-9 binding is landed (DB-14).** The transport block migration target is already a live substrate facility per [`../INVARIANTS.md`](../INVARIANTS.md) §E-9 and DB-14. Moving `transport shell { argv: [...] }` to per-target binding is mechanical, not novel substrate work."

### §2.2 Substrate state — corrected

- `src/v3/std/services.dag::Operation` — **EXISTS** (line 122 per design doc cite). Provides the caller-side effect-set pinning surface via `Operation.callable: CallableRef` + threaded signature on the referenced declaration.
- `src/v3/std/effects.dag` — provides EffectShape / OperationEffect / CompositionVerdict / compose_effects (substrate carriers).
- `Arrow.body` E-9 binding — landed per DB-14.

The carve doc's claim "4c: caller-side effect-set pinning carrier — NEW substrate introduction required" is **stale** relative to the locked design at `design-effect-enumeration-resource-threading.md`. Either the carve doc is wrong or the design doc is wrong; the design doc is the more recent locked authority and should be cited.

### §2.3 Migration scope (per design doc §6.2 atomic-migration shape)

Per design §6.2 "Why atomic is feasible (not aspirational)":
- **Lens body changes only in kind-classification dispatch** — current path (signature/body shape inference) retires; new path reads effect set from arrow signature directly + dispatches on `callable_inhabits` lookup.
- **`Operation` already exists** — no new carrier needed.
- **`Arrow.body` E-9 binding landed** — transport block migration is mechanical.

This makes C2 an **atomic migration** (single PR shippable), not a multi-stage substrate-introduction sequence. The 4-task split (4a/4b/4c/4d) from the carve doc reframes:
- **4a (resource-threading migration)**: bounded consumer migration; substrate exists.
- **4b (ambient metadata removal)**: bounded cleanup.
- **4c (caller-side effect-set pinning carrier)**: **substrate ALREADY exists** at `services.dag::Operation`; no new introduction required. Prior carve-doc claim was stale.
- **4d (full OperationEffect retirement)**: depends on 4a-4c convergence.

### §2.4 Recommendation — substrate-ready (revised)

**C2 is substrate-ready, not substrate-cliff.** Full carve-promotion to R3 — port-and-rewire (atomic migration shape per design §6.2). Same shape as C1 #81. Fold into Cluster F.

The earlier "(a) full carve-promotion (4c canvas authoring) vs (b) (γ) `.dag`-stub-form interim" disposition is **MOOT** — there's no 4c canvas to author since the carrier already exists. C2 just promotes alongside C1.

This corrects the audit's prior framing. C2's gate #82 #issuecomment-needed-disposition doesn't apply.

---

## §3. C3 — opt-in iteration parallelism via lens application (#95)

### §3.1 Substrate state — live ledger reading (corrected 2026-05-09)

**REVISION 2026-05-09 (round 2) per codex BLOCKING on PR #2363 sha `32afcd32`**: prior framing imported gate-status claims that misidentified the gate-number → carrier-landing mapping. Below is the **live ledger reading from `docs/r3-program-plan.md` §1.8 on main**.

**Live state of `src/v3/std/lens_application.dag` on main** (`git ls-tree origin/main -- src/v3/std/lens_application.dag` confirms; file content carries Slice A authority):

| Gate | Predicate | Status on main | Carrier in lens_application.dag |
|---|---|---|---|
| **#88** | `lens_application_carrier_landed` | **CONSUMER_LANDED** (Slice A receipt PR #2145) | `EnforcedApplication<Output, Budget>` + `IntrospectApplication<Output>` |
| **#89** | `section_ref_substrate_landed` | **CONSUMER_LANDED** (Slice A receipt PR #2145) | `SectionRef = DeclarationScope \| NodeScope` |
| **#90** | `lens_enforcement_carrier_landed` | **CONSUMER_LANDED** (Slice A receipt PR #2145) | parametric `LensEnforcement<Output, Budget>` + `EnforceableLens<Output, Budget>`; **per-lens data instances co-located with each lens land in Slice B** (per #90 Pass condition) |
| **#91** | `enforce_violation_routing_landed` | **DECLARED** — substrate routing surface landed PR #2145 (`DiagnosticSeverity = Error` + `EnforcedApplication.diagnostic_severity` + `LensEnforcement.violates`); **CONSUMER_LANDED requires the fold-pass consumer per design §10 step 2 — deferred to Slice B** |

**What is actually pending for Slice B** (per #90 + #91 Pass conditions on main):
- **Per-lens data instances** of `LensEnforcement<Output, Budget>` co-located with each lens (parallelism lens specifically required for #95)
- **Fold-pass consumer** for #91 violation routing per design `docs/design-lens-application-surface.md` §10 step 2

**Cascade chain for #95** per `docs/r3-structure.md`:164: "Pass requires parallelism lens BEHAVIORALLY COMPLETE (design §7 / §9 substantive-semantics cascade)." Plus parallelism-lens slice of Slice B authoring (per-lens LensEnforcement instance + parallelism violation routing).

**Lane-level dependency** per `docs/r3-structure.md`:61: the **T-Lens-Application-Surface lane** depends on **T-Lens-Behavioral-Parity COMPLETE** (lenses must be COMPLETE to produce useful structural facts on application sections), NOT only on parallelism (#81). T-LBP COMPLETE = complexity (#79) + cost (#80) + parallelism (#81 post-carve-promotion) + effect_enum (#82 post-carve-promotion). The Slice B authoring closure (per-lens enforcement projections for all in-R3 lenses + violation routing) gates on full T-LBP COMPLETE.

**Distinction**: lane-level vs gate-level dependencies for #95 specifically:
- **Lane-level (T-LAS lane closes — all #88-#95)**: gates on T-LBP COMPLETE (all 4 in-R3 lenses) per `r3-structure.md`:61.
- **Gate-level (#95 specific demo)**: gates on **parallelism-slice of Slice B** (parallelism `LensEnforcement` instance + parallelism violation routing) + **parallelism BEHAVIORALLY COMPLETE** (C1 #81). The full T-LBP-COMPLETE prerequisite is for full lane closure, not for #95-specific firing.

For #95 to fire as a demonstration gate, the parallelism slice of Slice B + parallelism behavioral completion are the load-bearing prerequisites. For the full T-LAS lane to close (all #88-#94 + #95), full T-LBP COMPLETE is required.

### §3.2 Substrate readiness — corrected scope

C3's gate #95 is a **demonstration gate** under the T-Lens-Application-Surface lane (per `r3-structure.md`:164 — "fourth worked example (design §4.4): opt-in cross-iteration parallelism via `Lens<Iteration-Independence>`"). Not a separate substrate carve — it's a worked-example demo of the shared lens-application surface lane.

**Cascade-gating chain** (live-ledger gate identifiers per main `r3-program-plan.md` §1.8):
1. **Slice A on main**: gates #88/#89/#90 CONSUMER_LANDED (parametric carriers + SectionRef + parametric LensEnforcement carrier in `src/v3/std/lens_application.dag`).
2. **Slice B substrate prerequisites for #95** (substrate-side):
   a. **Per-lens parallelism `LensEnforcement` instance** (#90 Pass condition's "per-lens data instances co-located with each lens land in Slice B" — parallelism slice specifically).
   b. **Fold-pass consumer for #91 violation routing** per design `docs/design-lens-application-surface.md` §10 step 2 (substrate routing surface landed PR #2145; consumer pending).
3. **Parallelism lens BEHAVIORALLY COMPLETE** (C1 #81 carve-promotion).
4. **#95 demonstration** lands as worked example via `apply_lens(parallelism, fn, Enforce { ... })`.

**Substrate prerequisite ordering**: Slice B substrate (2a + 2b) must land **before** #95 can fire as a worked-example demonstration gate. This makes carrier landing the prerequisite slice before #95, per codex BLOCKING ratification.

### §3.3 Recommendation — substrate-ready conditional on Slice B + C1

**Full carve-promotion to R3** with corrected scope: #95 cascade-gates on **(a) T-LAS Slice B landing** AND **(b) C1 parallelism lens BEHAVIORALLY COMPLETE**. Slice B is its own scoped lane work (within T-Lens-Application-Surface, not a separate carve). C3 promotes to R3-load-bearing as a worked-example demo gate; the substrate prerequisites (Slice B + C1) are tracked in their respective lanes.

---

## §4. Cluster F sequencing folder for #81/#82/#95

Per Director's structural framing, all three carve gates are **lens-producer-retirement** work. Fold into Cluster F (T-LP-Retirement):

| Sub-gate | Scope | Owner | Phase |
|---|---|---|---|
| **#81** parallelism lens | port `workflow_parallelism.rs` → `.dag` (M-sized lane; substrate-ready) | Substrate Mgr (warm-wolf-698) | Cluster F sub-phase α |
| **#82** effect_enum lens | atomic migration per design §6.2 (`services.dag::Operation` carrier exists; no new substrate intro) | Substrate Mgr (warm-wolf-698) | Cluster F sub-phase β (parallel-dispatchable with α) |
| **#95** opt-in iteration parallelism demo | lens_application demonstration (cascade post-#81 + T-LAS Slice B) | Verification Mgr (wise-bear-525) | Cluster F sub-phase γ (post-#81 + Slice B) |

Cluster F existing scope (#5/#6/#7 LP-retirement + #71 self-host trampoline) absorbs these as additional sub-gates per "lens-producer-retirement" structural framing.

---

## §5. Open questions for Director ratification

**Revised 2026-05-09 post-codex BLOCKING** — C2 (a vs γ) disposition is **MOOT** (carrier already exists per design doc; no canvas to author). Remaining open questions:

1. **Cluster F sequencing within itself**: #81 + #82 are both substrate-ready (parallel-dispatchable, no mutual substrate dep). #95 cascade-gates on #81 + T-LAS Slice B. PM recommends parallel for #81+#82, serial for #95 (cascade-gated).

2. **R4 carve doc dissolution**: amend `docs/r4-carve-out-routing.md` to remove C1/C2/C3 entries? Per Director's ratification language ("R4 carves dissolve") — yes; PM authors amendment in carve-promotion PR.

3. **T-LAS Slice B sequencing for #95**: Slice B (#91 per-lens LensEnforcement projection + violation routing per design §10 step 2) is a separate T-Lens-Application-Surface lane work item (not a separate carve). Does #95's R3 promotion require Slice B's R3 landing also explicitly tracked? PM recommends yes — both #91 + #95 are in the same lane-Mgr scope; tracking together in Cluster F sequencing folder.

---

## §6. Cycle-aggregate update

Director's velocity-to-zero update per #846 #issuecomment-4412330468 (revised 2026-05-09 post-codex BLOCKING):
- Original audit identified 4-6 bulk-dissolution events
- Carve-promotion adds: C1 walker port (+1 bulk event), C2 atomic migration (+1 bulk event; was +1 if (a) but disposition moot), C3 lens application demo (+1; cascade post-Slice-B + C1)
- **Updated estimate: 5-9 bulk events** for full PB-0 + carve-promotion R3 close

Each bounded substrate / cluster work; staffing-not-a-concern directive applies.

---

## §7. Out-of-scope follow-ups

Per Director's #846 #issuecomment-4412330468: **UNACCOUNTED entries in non-test census** need named retirement gates. PM follow-up grep authoring (separate audit doc) — for each unaccounted entry, surface the gap so Director can assign it a Cluster (F / K / M / V2-Retirement / new). Director: "If no Cluster fits, that's a structural gap requiring fresh substrate authoring."

This audit's scope is C1/C2/C3 only; UNACCOUNTED grep is a separate follow-up artifact (Task 13).

---

**End of audit.**
