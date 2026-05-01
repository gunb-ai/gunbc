---
status: PROPOSAL
owning_manager: Pure Bootstrap Manager (R2 → R3 continuation)
lane: T-Tier3-Dissolution sub-gate
authored: 2026-04-30 (PM deep-wolf-155 per PR #1319 Director ratification)
---

# R3 T-Tier3 Mirror Dissolution Perf Budget Worker Brief

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-04-30 by PM (deep-wolf-155) per PR #1319 ratification ask 4 ([gunbc#828 escalation](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356314413)).

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 1).

**Lane size:** S-M (~2-3 days).

**This is a planning artifact — not a dispatch order.** Worker dispatch is gated on T-Tier3-Dissolution mirror dissolution work landing first; see §"Dispatch preconditions" + §"STOP conditions". PB Manager re-reads at gate-clear.

## Scope

Author `tier3_mirror_dissolution_perf_within_budget` `.dag` `TestClaim` as a sub-gate of T-Tier3-Dissolution per Director ratification 2026-04-30 (PR #1319 amendment to [`docs/r3-structure.md`](../r3-structure.md) §"T-Tier3-Dissolution"). The gate verifies that retiring the four hand-Rust mirrors does not regress runtime performance beyond named thresholds.

Resolves R3 design challenge #7 ("Tier 3 mirror dissolution mechanics") per [`docs/r3-structure.md`](../r3-structure.md) §"Design challenges" by authoring the `.dag` TestClaim path. The narrative "≤2x slower acceptable" was Director-rejected as ambiguous; this brief commits to **enforced budget** with concrete thresholds.

The lane delivers:
1. Cargo bench fixtures comparing hand-Rust mirror invocation vs Evaluator-backed `.dag` invocation for each of the four retired mirrors.
2. The `.dag` TestClaim `tier3_mirror_dissolution_perf_within_budget` composing four per-mirror perf claims (one each for termination / computation / induction / effect-carrier).
3. CI integration so the gate fires on every PR touching T-Tier3-Dissolution surface.

## Out of scope

- **Mirror dissolution itself.** That's the [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) worker pack; this brief is a sibling perf-gate, not a duplicate.
- **Per-target perf budgets** (Rust vs Python vs Go). Tier 3 mirrors are Rust-internal; cross-target perf is T-CostLens-Composition territory.
- **Memory budgets.** Memory-peak is a separate framework deferred post-R3 per [`docs/design-lens-framework.md`](../design-lens-framework.md):99.

## Thresholds (Director-ratified 2026-04-30)

Per PR #1319 amendment to [`docs/r3-structure.md`](../r3-structure.md):

| Statistic | Threshold |
|---|---|
| **Median** | Evaluator-backed ≤ **2×** hand-Rust mirror |
| **p99** | Evaluator-backed ≤ **5×** hand-Rust mirror |

Both thresholds must hold per-mirror; gate fails if any single mirror exceeds either bracket.

**Rationale.** gunbc's substrate is decidable + closed-system; structural perf shouldn't be wildly different from hand-Rust. ≤2× median is the tight bound for typical case; ≤5× p99 allows for tail-latency variance from JIT warmup / cache effects without papering over real regressions. If real-world benchmarks land outside these brackets, escalate to Director (likely indicates substrate gap, not perf concern).

## Acceptance gate (`.dag`) — two-phase pattern

Codex BLOCKING review on PR #1331 sha `19dc267a` correctly flagged that comparing median(eval) against median(hand-Rust) requires both to exist simultaneously — exactly what T-Tier3-Dissolution forbids (parallel-implementation, INVARIANTS §P2). The gate is reframed as **two-phase: pre-dissolution baseline captured as frozen data; post-dissolution gate compares against the baseline data, not against any live mirror code**.

### Phase 1 — Baseline capture (pre-dissolution)

Before any mirror dissolution PR lands, a baseline-capture PR runs `criterion` benchmarks against the hand-Rust mirrors and **freezes the resulting timing data as JSON** at `src/v3/compiler/benches/tier3_baseline.json` (or sibling). The JSON file holds median + p99 in fixed nanoseconds for each of the four mirror benchmark groups; it is committed once and referenced thereafter.

After Phase 1 lands:
- The hand-Rust mirror code itself proceeds to dissolution per [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md).
- Only the **timing data** survives as `tier3_baseline.json`. The mirror code path is gone; only the measurement signature remains as data.

### Phase 2 — Post-dissolution gate (this brief's deliverables)

After all four mirrors are dissolved, `criterion` benchmarks run against the **`.dag`-evaluator path only** (the only path that exists post-dissolution). The gate fires by comparing measured timings against the frozen baseline data:

- Median(eval) ≤ **2×** median(baseline)
- p99(eval) ≤ **5×** p99(baseline)

`tier3_mirror_dissolution_perf_within_budget` is a `.dag` `TestSuite` composing **four per-mirror `TestClaim` rows** authored against existing `BehavioralObservation` substrate at `src/v3/std/verification.dag` (per `feedback_compiler_is_dag_processor` — no new substrate variant; structural composition over existing carriers).

### Per-mirror claims

| Claim | Mirror retired (Phase 1 baseline source) | Phase 2 bench (`.dag`-eval path only) |
|---|---|---|
| `tier3_termination_mirror_perf_within_budget` | `DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `ShrinkFactor`, `evidence_rank`, `merge_evidence` (was `dag.rs:628-790` per `r2-pure-bootstrap-manager.md:24`) | Representative descent-evidence merge over fixture corpus, evaluated through `.dag` body |
| `tier3_computation_mirror_perf_within_budget` | `SizeBound`, `RecursionShape`-related (was `dag.rs:839-915` per `r2-pure-bootstrap-manager.md:25`) | Representative recursion-shape derivation, evaluated through `.dag` body |
| `tier3_induction_mirror_perf_within_budget` | `RecursionShape`, `InductiveField`, `SubValueRelation` (was `dag.rs:916-980` per `r2-pure-bootstrap-manager.md:26`) | Representative sub-value-relation walk, evaluated through `.dag` body |
| `tier3_effect_carrier_mirror_perf_within_budget` | `dag/effects.rs` (216 LOC) + `compose_operation_effects` (105 LOC) per `r2-pure-bootstrap-manager.md:27` | Representative effect-composition over workflow corpus, evaluated through `.dag` body |

Each per-mirror claim is `BehavioralObservation { measured_median_ns ≤ 2 * baseline_median_ns AND measured_p99_ns ≤ 5 * baseline_p99_ns }` against the frozen `tier3_baseline.json` row for that mirror.

### Composition

`tier3_mirror_dissolution_perf_within_budget` evaluates `Conj` over the four per-mirror claims; **all four must hold**. Single-mirror failure fails the gate. Gate name matches Director ratification verbatim per PR #1319 amendment.

### Why this pattern preserves invariants

- **No parallel implementation** (INVARIANTS §P2): only `.dag`-eval path exists post-dissolution; baseline JSON is timing DATA, not parallel code authority.
- **Director thresholds preserved**: ≤2× median + ≤5× p99 ratification from PR #1319 still applies; the comparison point is just frozen data instead of live mirror.
- **Baseline noise concerns**: Phase 1 capture and Phase 2 measurement should run on the same canonical CI machine to keep comparison meaningful (hardware-stable). If CI infrastructure varies, escalate to Substrate Mgr.

## Deliverables

This lane delivers **Phase 2** (post-dissolution gate). Phase 1 (pre-dissolution baseline capture) is a sibling deliverable that lands BEFORE the dissolution PRs — it can be authored under this brief but its merge must precede T-Tier3-Dissolution mirror-dissolution PRs.

**Phase 1 deliverables** (sibling PR, must land before mirror dissolution):

0a. **`criterion` dev-dep added** in `src/v3/compiler/Cargo.toml`. Codex BLOCKING review on PR #1331 sha `1870104a` flagged that no `criterion` dependency currently exists in any `Cargo.toml`. Adding it is the first step of Phase 1.

0b. **Hand-Rust mirror benchmarks** at `src/v3/compiler/benches/tier3_mirror_perf.rs` (new file) — uses `criterion`. Per-mirror bench groups invoke the EXISTING hand-Rust mirror code (the `dag.rs:628-790`, `:839-915`, `:916-980`, `dag/effects.rs`, `workflow_idempotency.rs` sites). Statistically-sound sample sizes (`criterion` default = 100). **These benchmarks delete themselves alongside the mirror dissolution PRs** — they have no role post-Phase-1.

0c. **Frozen baseline data** at `src/v3/compiler/benches/tier3_baseline.json` — per-mirror median + p99 in fixed nanoseconds, captured by running 0b's bench harness on the canonical CI machine. Committed as data; serves as the comparison point for Phase 2. Survives mirror dissolution; mirror code does not.

**Phase 2 deliverables** (this lane's main work, post-dissolution):

1. **Eval-path bench fixtures** at `src/v3/compiler/benches/tier3_eval_perf.rs` (new file) — `criterion` benchmarks invoking the **`.dag`-evaluator path only** (Tier 3 std bodies executed via Evaluator). Per-mirror bench groups using the same fixture corpus as Phase 1's mirror benchmarks for like-for-like comparison.
2. **Stable benchmark inputs** — representative `DescentEvidence` / `SizeBound` / `SubValueRelation` / `EffectShape` fixtures committed under `src/v3/compiler/benches/tier3_fixtures/`. Inputs must be deterministic + version-pinned to avoid noise from corpus drift across PRs. Inputs are SHARED between Phase 1 and Phase 2 (so timings compare meaningfully).
3. **`.dag` `TestClaim`** authored at `src/v3/std/verification.dag` (or sibling) — single suite + four sub-claims comparing measured Phase-2 timings against frozen `tier3_baseline.json` row per mirror; compose via existing `Conj` over `BehavioralObservation` per `feedback_compiler_is_dag_processor` (no new substrate variant).
4. **CI wiring** — Phase 2 bench runs on PRs touching the canonical `.dag` authorities + Evaluator path:
   - **`.dag` authorities** (canonical): `dsl/std/{termination,computation,induction,effects}.dag` (root authority for these std blocks)
   - **`src/v3/std/` substrate twins** (currently exist as v3-side files for these four blocks; verified live 2026-04-30): `src/v3/std/{termination,computation,induction,effects}.dag`
   - **Evaluator surface**: `src/v3/compiler/src/v3_eval/` (or wherever Evaluator code lives at gate-clear)
   - **Frozen baseline**: `src/v3/compiler/benches/tier3_baseline.json` (read-only data; CI rejects edits)
   - Gate fails with explicit per-mirror diagnostic naming which budget bracket (median ≤2× baseline / p99 ≤5× baseline) was breached.

   Note: post-dissolution, the Rust mirror sites (`dag.rs:628-790`, `:839-915`, `:916-980`, `dag/effects.rs`, `workflow_idempotency.rs`) no longer exist as mirror code — only as historical line references in the baseline metadata. CI watches the Evaluator path, not the deleted mirror paths.

## Dependencies

Per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" + §"Dependency DAG":

1. **Phase 1 baseline lands BEFORE T-Tier3-Dissolution mirror dissolution PRs.** Strict temporal ordering: Phase 1 captures hand-Rust timing data → mirror dissolution PRs land → Phase 2 fires gate against frozen baseline. Reverse order makes baseline capture impossible (mirror code is gone).
2. **T-Tier3-Dissolution mirror dissolution PRs landed** before Phase 2 dispatches. Phase 2 measures the `.dag`-eval path; the Rust mirrors must already be retired so that benchmark scope is unambiguous (only one path exists).
3. **R2-Evaluator landed.** Phase 2 measurement requires running `.dag` bodies via Evaluator. T-Evaluator close is the upstream gate.
4. **No precondition on `criterion`** — adding the dev-dep is part of Phase 1 deliverable 0a above; not assumed as already-present.

## Dispatch preconditions

**Phase 1** (baseline capture; sibling PR) dispatches when:
- R2-Evaluator readiness signal received OR R2-close stable (Phase 1 measures hand-Rust mirrors, not the Evaluator path; Evaluator readiness is informational, not gating for Phase 1).
- PB Manager has reviewed the brief and confirmed the canonical CI machine for baseline capture.

**Phase 2** (this lane's main work) dispatches when:
- Phase 1 PR has merged on `main` with `tier3_baseline.json` committed.
- All four Tier 3 mirror dissolution PRs have landed on `main` (per [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md)).
- R2-Evaluator readiness signal received from Evaluator Manager.
- This brief's design has been re-reviewed by PB Manager at gate-clear.

## STOP conditions

Worker STOPs and PINGs (canonical output: docs-only audit PR, per `feedback_worker_stall_diagnosis` substrate-gap-stall pattern) if:

1. **Phase 1 must precede dissolution; reverse order is impossible.** If T-Tier3-Dissolution mirror PRs land before Phase 1 baseline capture, baseline can no longer be measured (mirror code is gone). STOP and escalate to PB Manager + Substrate Mgr — the perf gate becomes unrecoverable without a re-author of the mirror or a structural reframe of the threshold (relative → absolute).
2. **Phase 2 bench results are >10× over baseline** — likely indicates substrate gap (e.g., Evaluator runtime is missing critical optimization, NOT a perf-budget concern). Escalate to Substrate Mgr + Evaluator Mgr cross-program coordination.
3. **Phase 1 captured but baseline JSON shape is unstable** (e.g., per-CI-machine variance >20% run-to-run) — tooling concern. Surface to Substrate Mgr; the CI machine question may need locking before Phase 2 measurement is meaningful.
4. **Hand-Rust mirror has additional callers post-dissolution.** If T-Tier3-Dissolution leaves any consumer reaching the dissolved Rust path (e.g., a stale internal call site), Phase 2 measurement scope is ambiguous (parallel-implementation residue). STOP and route the leftover to T-Tier3-Dissolution before Phase 2.

## Discipline

**Per INVARIANTS §P2 (no parallel implementations):** the two-phase pattern explicitly does NOT keep the hand-Rust mirror alive for benchmarking. Phase 1 captures timing data BEFORE dissolution; mirror code itself is gone after dissolution. Only the JSON measurement signature survives — that's data, not parallel authority.

**Per `feedback_construction_over_ratchets`:** the gate is structural-acceptance for the dissolution work; not a heuristic perf-warning. If Phase 2 breaches budget against frozen baseline, the dissolution stands but the lane fails — escalate to Director for either threshold renegotiation OR substrate-gap investigation (Evaluator missing optimization that the Rust mirror had).

**Per `feedback_no_textual_enforcement_bridges`:** the gate fires on `.dag` `TestClaim` evaluation, NOT on grep/regex over bench output. CI parses bench JSON output and feeds the structural comparison (`measured_median ≤ 2 * baseline_median`) into the TestPredicate.

**Per `feedback_structural_perf_tests`:** prefer testing performance via operation counts (lens-fold visit count) where structurally possible. The mirror-perf budget is wall-clock because the question IS "does running `.dag` body cost the same as the inlined Rust mirror it replaced," which is wall-clock by construction. Operation-count tests live elsewhere (T-Verification-L4-L7-Direct).

**Per `feedback_no_generated_code_on_disk`:** `tier3_baseline.json` is captured DATA, not generated code. It does not count against SG-0 census; it is read-only after capture (CI rejects edits unless under a baseline-recapture PR with explicit Director approval).

## Cross-refs

- Parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"T-Tier3-Dissolution"
- Sibling brief: [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) — the dissolution work itself
- PB Manager scope: [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owned deliverables"
- Director ratification: [PR #1319](https://github.com/gunb-ai/gunbc/pull/1319) (R3 amendment)
- Closure ledger: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §"Pure Bootstrap Manager"
- INVARIANTS: §P5 (dispatch-discipline)
