# Canvas — Q-PerfWithinBaseline (REVISED — supersedes wrong-shape original)

**Authority**: Director disposition at gunb-ai/gunbc#828 #issuecomment-4403265220 — supersession authorized after BLOCKING at PR #2209 c#4403246233 verified VALID. Original canvas at `docs/proposals/q-perf-within-baseline-canvas.md` (merged via PR #2209) is **SUPERSEDED-BY this revised canvas**; original retained as historical record per durable-decision-record discipline.

**Status**: **canvas — DRAFT 2026-05-08 (REVISED)**; PROPOSAL maturation pending Director path-call ratification.

**Sub-issue**: gunb-ai/gunbc#2204 (Substrate T-Tier3 PerfWithinBaseline TestPredicate variant); cross-routed from PB Mgr #2074 c#4403061394.

## What this canvas resolves

Original canvas conflated **symbolic SymbolicCost** (cost-lens substrate) with **wall-clock measured median/p99 baseline** (T-Tier3 perf gate substrate) — two distinct fact-shapes. The Mgr lean Q1(b) `baseline_ref: DeclarationRef → data baseline_X: SymbolicCost` addressed substrate-precedent (`LensOutputEquals` pattern) but missed the consumer-side requirement.

**Substrate-grep at canonical authority** (`docs/r3-structure.md:225` + `:461`):

> **C1 — Tier-3 mirror dissolution perf budget** sub-gate of T-Tier3-Dissolution: `tier3_mirror_dissolution_perf_within_budget` with thresholds **≤2× median, ≤5× p99**.

> **DECISION (Director-locked 2026-04-28):** Either measurable as a `.dag` TestClaim (`tier3_mirror_dissolution_perf_within_budget` with explicit numeric threshold) OR explicitly post-R3 with no in-R3 perf gate. ... Recommended path: **explicitly post-R3** unless someone authors the perf-budget claim with concrete numbers and tooling. R3 deliverable is structural close; perf is downstream.

The gate measures **wall-clock perf at runtime** with ratio thresholds — a different substrate axis from substrate-time `data X: SymbolicCost` declarations.

## The strategic question (revised Q1)

Per Director-locked 2026-04-28 (`r3-structure.md:461`), there are TWO paths for T-Tier3 perf budget:

### Path P1 — Author full perf-budget substrate IN-R3

**Scope**: introduce wall-clock-measurement carrier as new substrate-fact (genuinely-novel; canvas-finding taxonomy pattern 5):

- New substrate type `PerfBaselineMeasurement { median_ns: Int, p99_ns: Int }` (or analogous)
- Runtime-resolution path for measurement (where do measured values come from? fixture-storage? captured-at-test-runner-time? CI-collected baseline?)
- Ratio-comparison op composition (\`≤2× median\` is real-number ratio; existing `ComparisonOp` may not compose; possibly new `RatioComparisonOp`)
- Multi-slice authoring (carrier introduction + runtime-resolution + comparison-op + cementing fixture + integration with T-Tier3 dispatch)

**Cycle cost**: multi-slice in-R3 effort; substantial substrate-fact-introduction not previously budgeted in R3 plan; uncertain whether achievable in R3 horizon given current 5-orthogonal-dependency picture for T-LBP cementing close.

**Pro**: keeps T-Tier3 R-4 perf gate in R3 closure surface; structural completeness.
**Con**: multi-slice substantial scope; risks slipping R3 close; Director-locked recommendation explicitly leans against this path absent concrete tooling/numbers authorship.

### Path P2 — Explicitly post-R3 (Director-locked 2026-04-28 RECOMMENDED PATH)

**Scope**: explicitly defer perf-budget claim to post-R3; in-R3 deliverable for T-Tier3 is **structural close only** (mirror retirement; consumer count → 0; SG-0 delta reported per `r3-structure.md:196` lane row). T-Tier3 R-4 perf gate is post-R3-carved per Director-locked decision.

**Concrete actions if P2 ratifies**:
1. **Close #2204 sub-issue as superseded-by-deferral** — `PerfWithinBaseline` TestPredicate variant authoring is post-R3 work, not in-R3 substrate.
2. **Notify PB Mgr (#2074)** that #2138 worker dispatch chain is post-R3 per Director-locked deferral; PB R-4 prereq for #2085 dissolves to "post-R3" status (R-4 is not an R3-load-bearing gate per the explicit `r3-structure.md:461` framing).
3. **Update §10.3 capability-register row** for Q-PerfWithinBaseline canvas: SUPERSEDED-by-revised + revised disposition POST-R3-DEFERRED.
4. **Preserve original canvas as historical record** with SUPERSEDED-BY header pointing to this revised canvas.

**Cycle cost**: zero in-R3 substrate effort; T-Tier3 lane closes structurally; perf substrate is genuinely-post-R3 work.

**Pro**: matches Director-locked 2026-04-28 recommended path; respects R3 horizon; structural close = R3 deliverable per `r3-structure.md:461` ("R3 deliverable is structural close; perf is downstream").
**Con**: cross-Mgr signal: PB needs to absorb #2138 worker dispatch deferral; may surface R3-close-shape question if T-Tier3 lane has implicit perf-gate expectations from other consumers.

## Trigger-condition state at HEAD (CORRECTED 2026-05-08 per Director c#4403297623)

Director-locked recommendation at `r3-structure.md:461` reads: *"explicitly post-R3 unless someone authors the perf-budget claim with concrete numbers and tooling."* My initial supersession-canvas Mgr-lean reasoning #1 ("path-trigger condition for P1 is not met") was **structurally wrong** per Director's grep at c#4403297623. Existing tooling at HEAD substantially meets the trigger:

| Artifact | Path | Status |
|---|---|---|
| Perf-budget worker brief | `docs/briefs/r3-pb-tier3-perf-budget-worker.md` | landed |
| Readiness matrix | `docs/audit/c1-tier3-perf-budget-readiness-matrix.md` | landed |
| Bench harness | `src/v3/compiler/benches/tier3_mirror_perf.rs` | merged |
| Baseline capture procedure | `docs/audit/c1-tier3-baseline-capture-procedure.md` | landed |
| Canonical bench host decision matrix | `docs/audit/c1-r3-canonical-bench-host-decision-matrix.md` | landed |

Plus concrete thresholds explicit (≤2× median, ≤5× p99); multi-run capture discipline authored (N=3 minimum, N=5 preferred); option matrix for canonical bench host. **Trigger-condition is effectively met.** P1 (in-R3 substrate authoring) is bridging existing tooling to substrate — NOT multi-slice greenfield as I originally framed. Smaller scope than canvas-finding-pattern 5 ("genuinely-novel substrate-fact-introduction") implies; closer to compositional-extension over existing tooling-substrate boundary.

## Mgr lean (REVISED — path-call deferred to PB Mgr cross-Mgr coordination)

Per Director disposition at c#4403297623, **path-call is NOT Substrate-Mgr-leanable as a free-standing call** — it depends on PB Mgr's load-bearing R-3 (canonical CI bench host) + R-7 (`tier3_baseline.json` baseline-capture path) decisions. PB Mgr's audit response at #828 c#4403059897 surfaced both as pending.

**Cross-Mgr coordination required**:
- If PB Mgr resolves R-3 + R-7 cleanly + chooses to author the in-R3 substrate-variant integration → **P1 is the natural path**
- If PB Mgr defers R-3/R-7 OR chooses a path that doesn't require the substrate-variant in R3 → **P2 is the natural path**

**Cross-Mgr surface (this canvas's actionable next step)**: ping PB Mgr (#2074) asking for R-3/R-7 disposition + their lean on whether P1 in-R3 substrate-variant authoring fits in PB lane scope OR P2 defer is the right call. Path-call ratification follows PB Mgr's surface.

**Substrate Mgr provisional lean (post-PB-cross-Mgr-coord)**:
- If PB Mgr leans P1: Substrate Mgr concurs (existing tooling + bridging to substrate is bounded scope; tooling-substrate-bridge does NOT trigger pattern-5 P1 procedure since carrier-shape composes existing tooling at HEAD).
- If PB Mgr leans P2: Substrate Mgr concurs (R3 horizon + structural-close priority are clean reasons; no longer rests on disproven trigger-condition framing).

## Director ratification ask (REVISED)

**Path call deferred to PB Mgr cross-Mgr coordination outcome**:

1. Substrate Mgr surfaces to PB Mgr (#2074) with corrected trigger-condition framing + R-3/R-7 disposition ask + P1/P2 lean ask.
2. PB Mgr responds with R-3/R-7 disposition + their P1/P2 lean.
3. Director ratifies path-call based on cross-Mgr alignment between Substrate + PB Mgr leans.

**Substrate Mgr commits to surface to PB this turn** with the corrected framing.

If P2 ratifies:
- Close #2204 as superseded-by-deferral
- Notify PB Mgr (#2074) for #2138 dispatch chain disposition
- Update §10.3 capability-register row + r3-program-plan §10.3 row for T-Tier3 R-4 explicit-post-R3 status
- Original canvas at `docs/proposals/q-perf-within-baseline-canvas.md` retained with SUPERSEDED-BY header

If P1 ratifies:
- Reset canvas-tier P1 procedure (DAG-ancestor / coproduct-vs-coordinate / primitive-vs-lens-extensible) on `PerfBaselineMeasurement` carrier
- Multi-slice authoring sequence
- Substrate-shape canvas extension on runtime-resolution path

## History (auto-merge race + cycle-tier discipline)

**Auto-merge race**: original canvas merged at 04:02:35Z; codex BLOCKING surfaced at 04:02:43Z (~8s window). Director's disposition at c#4403265220 declined sit-window patch per `feedback_construction_over_ratchets` — structural fix is grep-verify-consumer-side at canonical authority doc, not process-discipline-patch.

**Two-axis verification discipline gap** (Director-tier self-flag at c#4403265220): canvas authoring + ratification both verified substrate-precedent (`LensOutputEquals` pattern) but neither verified consumer-side requirement (`r3-structure.md:225` + `:461` gate semantics). Memorialized as 5-row canvas-finding-pattern triage **precondition**: triage applies ONLY when consumer-side requirement is grep-verified at canonical authority doc.

**Cycle position**: 5th discipline-failure in 2026-05-08 audit cycle (after complexity-lens brief BLOCKING, Slice 5 dispatch substrate-gap, ε-path tier-disambiguation, Q5 strict-mirror triage, Q6 substrate-precedent-without-consumer-side). Pattern: substrate-grep discipline at canonical authority doc has multiple pitfalls; triage taxonomy refines per cycle's findings.

## Framework discipline anchors

- **`feedback_substrate_grep_before_authoring`**: violated at canvas-authoring layer (consumer-side grep missed); discipline-anchor surfaces from this BLOCKING + revision.
- **`feedback_canvas_finding_taxonomy`**: 5-row table needs precondition "consumer-side requirement grep-verified at canonical authority doc" per Director's c#4403265220 refinement.
- **`feedback_construction_over_ratchets`**: structural fix (grep discipline) > process-discipline-patch (sit-window); ratify via P2 per Director-locked recommendation.
- **`feedback_dispatch_dont_preask_symmetric`**: Director's two-axis verification gap symmetric to my prior cycle violations; cross-tier discipline.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-08 per Director disposition at gunb-ai/gunbc#828 #issuecomment-4403265220. Supersedes `docs/proposals/q-perf-within-baseline-canvas.md` (merged via PR #2209; retained as historical record).
