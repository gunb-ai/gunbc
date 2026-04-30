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

## Acceptance gate (`.dag`)

`tier3_mirror_dissolution_perf_within_budget` is a `.dag` `TestSuite` composing **four per-mirror `TestClaim` rows** — one per retired mirror, each authored against existing `BehavioralObservation` substrate at `src/v3/std/verification.dag` (per `feedback_compiler_is_dag_processor` — no new substrate variant; structural composition over existing carriers).

### Per-mirror claims

| Claim | Mirror retired | Bench fixture |
|---|---|---|
| `tier3_termination_mirror_perf_within_budget` | `DescentEvidence`, `PositiveDescentAmount`, `ProportionalDivisor`, `ShrinkFactor`, `evidence_rank`, `merge_evidence` (was `dag.rs:628-790` per `r2-pure-bootstrap-manager.md:24`) | Representative descent-evidence merge over fixture corpus |
| `tier3_computation_mirror_perf_within_budget` | `SizeBound`, `RecursionShape`-related (was `dag.rs:839-915` per `r2-pure-bootstrap-manager.md:25`) | Representative recursion-shape derivation |
| `tier3_induction_mirror_perf_within_budget` | `RecursionShape`, `InductiveField`, `SubValueRelation` (was `dag.rs:916-980` per `r2-pure-bootstrap-manager.md:26`) | Representative sub-value-relation walk |
| `tier3_effect_carrier_mirror_perf_within_budget` | `dag/effects.rs` (216 LOC) + `compose_operation_effects` (105 LOC) per `r2-pure-bootstrap-manager.md:27` | Representative effect-composition over workflow corpus |

### Composition

`tier3_mirror_dissolution_perf_within_budget` evaluates `Conj` over the four per-mirror claims; **all four must hold**. Single-mirror failure fails the gate. Gate name matches Director ratification verbatim per PR #1319 amendment.

## Deliverables

1. **Cargo bench fixtures** at `src/v3/compiler/benches/tier3_mirror_perf.rs` (new file) — uses `criterion` (existing dev-dep). Per-mirror bench groups; statistically-sound sample sizes (`criterion` default = 100).
2. **Stable benchmark inputs** — representative `DescentEvidence` / `SizeBound` / `SubValueRelation` / `EffectShape` fixtures committed under `src/v3/compiler/benches/tier3_fixtures/`. Inputs must be deterministic + version-pinned to avoid noise from corpus drift across PRs.
3. **`.dag` `TestClaim`** authored at `src/v3/std/verification.dag` (or sibling) — single suite + four sub-claims; compose via existing `Conj` over `BehavioralObservation` per `feedback_compiler_is_dag_processor` (no new substrate variant).
4. **`criterion` dev-dep added.** Codex BLOCKING review on PR #1331 sha `1870104a` flagged that no `criterion` dependency currently exists in any `Cargo.toml`. Adding `criterion` as `[dev-dependencies]` in `src/v3/compiler/Cargo.toml` is part of THIS lane's deliverables, not a precondition. The worker's first commit adds the dep; bench fixtures + harness follow.
5. **CI wiring** — bench runs on PRs touching the canonical std authorities AND the Rust mirror sites being measured:
   - **`.dag` authorities** (canonical): `dsl/std/{termination,computation,induction,effects}.dag` (root authority for these std blocks)
   - **`src/v3/std/` substrate twins** (currently exist as v3-side files for these four blocks; verified live 2026-04-30): `src/v3/std/{termination,computation,induction,effects}.dag`
   - **Rust mirror sites being retired**: `src/v3/compiler/src/dag.rs` (mirror line ranges per `r2-pure-bootstrap-manager.md:24-26`), `src/v3/compiler/src/dag/effects.rs`, `src/v3/compiler/src/workflow_idempotency.rs`
   - Gate fails with explicit per-mirror diagnostic naming which budget bracket (median ≤2× / p99 ≤5×) was breached.

## Dependencies

Per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" + §"Dependency DAG":

1. **T-Tier3-Dissolution mirror dissolution landing.** The four mirrors must actually be dissolved before perf measurement makes sense (you can't measure overhead vs nothing). Per [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md): per-mirror dissolution PRs (worker pack already authored).
2. **R2-Evaluator landed.** Perf measurement requires running `.dag` bodies via Evaluator. T-Evaluator close is the upstream gate.
3. **No precondition on `criterion`** — adding the dev-dep is part of deliverable #4 above; not assumed as already-present.

## Dispatch preconditions

Worker dispatches when:
- All four Tier 3 mirror dissolution PRs have landed on `main` (or are in flight in the same wave).
- R2-Evaluator readiness signal received from Evaluator Manager (or PR-B/C/D/E cadence has converged sufficiently).
- This brief's design has been reviewed by PB Manager (R3 continuation) at gate-clear.

## STOP conditions

Worker STOPs and PINGs (canonical output: docs-only audit PR, per `feedback_worker_stall_diagnosis` substrate-gap-stall pattern) if:

1. **Bench results are >10× off** — likely indicates substrate gap (e.g., Evaluator runtime is missing critical optimization, NOT a perf-budget concern). Escalate to Substrate Mgr + Evaluator Mgr cross-program coordination.
2. **Mirror dissolution doesn't land in the same wave** — perf-gate has nothing to measure against; STOP and re-dispatch when prerequisites land.
3. **`criterion` benchmarks are flaky in CI** (>20% run-to-run variance on the same input) — surface to Substrate Mgr; tooling concern, not perf-budget concern.

## Discipline

**Per `feedback_construction_over_ratchets`:** the gate is structural-acceptance for the dissolution work; not a heuristic perf-warning. If bench breaches budget, the dissolution PR is rejected — not papered over with threshold relaxation.

**Per `feedback_no_textual_enforcement_bridges`:** the gate fires on `.dag` `TestClaim` evaluation, NOT on grep/regex over bench output. CI parses bench JSON output and feeds into TestPredicate.

**Per `feedback_structural_perf_tests`:** prefer testing performance via operation counts (lens-fold visit count) where structurally possible. The mirror-perf budget is wall-clock because the question IS "does running `.dag` body cost the same as the inlined Rust mirror it replaced," which is wall-clock by construction. Operation-count tests live elsewhere (T-Verification-L4-L7-Direct).

## Cross-refs

- Parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"T-Tier3-Dissolution"
- Sibling brief: [`r2-pb-tier3-mirror-dissolution-workers.md`](r2-pb-tier3-mirror-dissolution-workers.md) — the dissolution work itself
- PB Manager scope: [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owned deliverables"
- Director ratification: [PR #1319](https://github.com/gunb-ai/gunbc/pull/1319) (R3 amendment)
- Closure ledger: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §"Pure Bootstrap Manager"
- INVARIANTS: §P5 (dispatch-discipline)
