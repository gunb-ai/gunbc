---
status: PROPOSAL (draft pre-#2252-merge; awaiting PB Mgr ACK)
owning_manager: Pure Bootstrap Manager (R2 → R3 continuation)
lane: T-Tier3-Dissolution consumer slice (post-#2204 substrate slice)
authored: 2026-05-08 (crisp-swift-433 lane tracker per #2074 c#4405437035 + c#4405490891 scope-expansion ratification)
---

# T-Tier3-Dissolution — Consumer-Slice Worker Brief

**Status:** PROPOSAL. Authored 2026-05-08 by tracker-promoted-to-author crisp-swift-433 per PB Mgr scope-expansion at #2074 c#4405437035 + c#4405490891. **Dispatch-gated on**: (a) #2252 substrate-slice strip + merge (carrier + variant + cementing land cleanly per Substrate Mgr c#4405471313); (b) PB Mgr ACK on this brief.

**Owning manager:** Pure Bootstrap Manager (warm-dove-618 #2074).

**Lane size:** M (~3-5 days; 4 deliverables, two cross-component touch).

**Scope authority:** Director ratification at #828 c#4403480005 (P1 + R-3 + R-7 + #2204 acceptance gates) + PB Mgr scope-expansion at #2074 c#4405437035 + c#4405490891 (consumer-slice 4-item carve, option-B aligned with Substrate Mgr).

## Goal

Wire the substrate-tier `PerfWithinBaseline` TestPredicate (landed via #2204 / PR #2252 stripped form) into the T-Tier3 R-4 perf-budget gate (`tier3_mirror_dissolution_perf_within_budget`), demonstrating end-to-end gate clearing against the Phase-1 baseline captured per ratified R-7 procedure on the ratified R-3 host.

## Deliverables

### 1. Evaluator-side runtime invariant impl (`eval_perf_within_baseline`)

**File:** `src/v3/compiler/src/test_runner.rs`.

**Shape** (per #2204 acceptance gate semantics + §225 thresholds):
- Add dispatch arm `"PerfWithinBaseline" => self.eval_perf_within_baseline(claim, &payload)` in `TestRunner::evaluate` predicate match.
- Implement `fn eval_perf_within_baseline(&self, claim: &TestClaimValue, payload: &[FieldValue]) -> ClaimResult`:
  - Destructure 3-field payload: `subject: DeclarationRef`, `comparator: ComparisonOp`, `baseline_ref: DeclarationRef`.
  - Resolve both `DeclarationRef`s to `PerfBaselineMeasurement` data declarations (use existing `resolve_declaration_ref_id` helper).
  - Apply ratified ratio budget: `median_bound = baseline.median_ns × 2`; `p99_bound = baseline.p99_ns × 5` (saturate-on-overflow → Fail with explicit reason; do NOT silently wrap).
  - Apply `comparator` to both axes; both must satisfy → `Pass`. Otherwise `Fail` with both subject/threshold values surfaced for triage.
- Helper `fn perf_baseline_measurement(&self, decl_id: DeclarationId, role: &str) -> Result<PerfMeasurement, String>`: structural resolve of `{ median_ns, p99_ns }` from declaration's data record fields.

**Note on jolly-dove-416 prior art:** child session's stripped-from-#2252 `eval_perf_within_baseline` impl is structurally a fit; reuse-with-attribution is acceptable (option-B carve places this code here, not there). Worker should cross-check the stripped form against this brief's spec — not re-invent wholesale.

**Acceptance:** unit tests in `#[cfg(test)] mod` inside `src/v3/compiler/src/test_runner.rs` exercising the four cases — pass-when-under-budget, fail-on-median-over, fail-on-p99-over, fail-on-overflow. Per `TESTING.md` unit-vs-integration discipline, this is a narrow crate-internal helper; tests live alongside source. Integration coverage in `tests/integration/test_runner_test.rs` is reserved for the pipeline-as-unit gate-clearing path exercised through deliverable §3 (the `tier3_mirror_dissolution_perf_within_budget` TestClaim end-to-end).

### 2. Bench-harness integration with `tier3_mirror_perf.rs`

**File:** `src/v3/compiler/benches/tier3_mirror_perf.rs` + new capture-and-emit driver.

**Shape:**
- Existing skeleton has 4 criterion benches (termination / computation / induction / effect-carrier) — keep as-is for Phase-1 baseline measurement of hand-Rust mirrors.
- Add **Phase-2 bench** (sibling group `tier3_mirror_phase2`): same 4 benches, but invoking the `.dag`-Evaluator path via `TestRunner` end-to-end on a fixture `.dag` program that exercises each mirror's signature operation through the evaluated std body. Phase-1 baseline measures the Rust path; Phase-2 measures the dissolved-mirror path; the gate compares.
- Add a small CLI under `src/v3/compiler/src/bin/` (or analogous) — `tier3_baseline_capture` — that runs `cargo bench tier3_mirror_phase1 -- --measurement-time` per ratified R-7 procedure (N=5, median-of-medians, max-p99-across-runs), serializes to `tier3_baseline.json` schema (per `docs/audit/c1-tier3-baseline-capture-procedure.md`), and prints the JSON.
- Phase-2 measurement path emits same JSON schema; both consumed by §3 below.

**Capture invocation** (per ratified R-7, runs only on `ubicloud-standard-2` per ratified R-3): `cargo bench --bench tier3_mirror_perf -- --N=5 --output tier3_baseline.json`. Capture-once-and-commit; recapture only on Director-approved trigger conditions.

**Acceptance:** `tier3_baseline.json` lands at `src/v3/compiler/benches/tier3_baseline.json` (or `docs/audit/`) with all 4 mirror groups populated.

### 3. `tier3_mirror_dissolution_perf_within_budget` claim authoring

**Files:** new `src/v3/std/tier3_perf_budget.dag` (or extension to existing T-Tier3 module).

**Shape** (per `r3-structure.md` §225 + Director c#4403480005 acceptance gate 6):

```dag
data tier3_termination_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_computation_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_induction_baseline:   PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_effect_carrier_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }

data tier3_termination_phase2_measurement: PerfBaselineMeasurement = { median_ns: <Phase-2-captured>, p99_ns: <Phase-2-captured> }
// ... three more

data tier3_mirror_dissolution_perf_within_budget: TestClaim = TestClaim {
  predicate: PerfWithinBaseline {
    subject: DeclarationRef("tier3_termination_phase2_measurement"),
    comparator: Le,
    baseline_ref: DeclarationRef("tier3_termination_baseline"),
  },
  // ... composed with the other three predicates per existing TestClaim composition pattern
}
```

**Composition note:** per §225 ("≤2× median, ≤5× p99 thresholds"), the runtime invariant impl in §1 applies the ratio. The .dag claim declares `comparator: Le`; runtime applies the budget multiplier. Composition of the 4 per-mirror claims into one suite-level claim matches existing TestClaim composition patterns.

**Acceptance:** `cargo test -p v3-compiler tier3_mirror_dissolution_perf_within_budget` passes against captured baselines + Phase-2 measurements. Claim is registered in §1.8 ledger as **CONSUMER_LANDED**.

### 4. Cementing receipt: end-to-end T-Tier3 R-4 gate clearing

**Shape:**
- After mirror-dissolution PRs land (the 4 per-mirror retirement workers in PB canvas, separate dispatch chain): re-run Phase-2 capture. Gate clears if all 4 mirror Phase-2 measurements satisfy `≤ baseline × {2, 5}`. Per PB Mgr disposition at PR #2254 review, this measurement run is a separate trigger post-#2204-consumer slice merge (deliverables 1-3 author against mock-shape; deliverable 4 measurement waits for per-mirror retirement chain).
- Receipt artifact: `docs/audit/c1-tier3-perf-budget-receipt.md` — captures Phase-1 baseline rows, Phase-2 measurement rows, gate-clearing status per mirror. Carries explicit "execute when per-mirror dispatch completes" note tying the measurement run to retirement-chain landing.
- §1.8 ledger row update: `tier3_mirror_dissolution_perf_within_budget` flips DECLARED → PASSING with PR-link evidence.

**Acceptance:** receipt doc lands, ledger updated, T-Tier3 R-4 gate flips green in `r3-program-plan.md` §3 lane status table for T-Tier3-Dissolution.

## Dispatch preconditions (HARD)

1. **#2252 substrate-slice strip + merge** — carrier (`PerfBaselineMeasurement`) + variant (`PerfWithinBaseline` 3-field) + substrate-tier shape-attestation cementing tests on `main`. Verify via grep at HEAD before worker dispatch:
   - `grep -n "type PerfBaselineMeasurement" src/v3/std/substrate.dag` — must match
   - `grep -n "PerfWithinBaseline" src/v3/std/verification.dag` — must match
2. **PB Mgr ACK on this brief** (#2074).
3. **Phase-2 brief authoring runs in parallel with per-mirror retirement workers' mock-shape stability** (PB Mgr disposition at PR #2254 review): deliverables 1-3 author against the mock-shape per-mirror Evaluator path without waiting for per-mirror retirement landing. Deliverable 4 measurement run executes when the retirement chain completes (separate trigger, post-#2204-consumer slice merge — receipt artifact in §4 carries an explicit "execute when per-mirror dispatch completes" note).

## STOP conditions

- If #2252 substrate carrier shape changes post-merge (e.g., adds fields beyond `{ median_ns, p99_ns }`): STOP, surface to PB Mgr, re-author this brief.
- If the `PerfWithinBaseline` 3-field shape changes (e.g., adds `tolerance_factor` field that overrides §225 thresholds at the predicate site): STOP, escalate; budget thresholds are Director-locked at §225.
- If a per-mirror retirement worker discovers Evaluator-API gap (e.g., `kernel_algebra_profile` map read-path needs substrate work): surface to PB Mgr per #2085 body's "Substrate-Mgr canvas" handoff line; do NOT absorb substrate-shape work into this consumer slice.
- If the bench-harness Phase-2 path requires a new `TestRunner` capability beyond `eval_perf_within_baseline` (e.g., fixture-program execution-time measurement integration): STOP, escalate to Verification Mgr — that's verification-runtime substrate, not this slice.

## Out of scope

- Per-mirror retirement worker briefs (the 4 mirror dissolution PRs themselves). Those are PB Mgr canvas territory per #2085 body.
- `PerfBaselineMeasurement` carrier shape extensions (Substrate Mgr authority).
- `PerfWithinBaseline` predicate shape extensions (Substrate Mgr authority).
- §225 budget threshold changes (Director authority).
- SG-0 census ratchet behavior — substrate-tier work, no consumer-slice impact expected; verify-and-flag if bootstrap_generated regen ratchets.

## Cross-Mgr coordination points

- **Substrate Mgr (#2068)** — confirm carrier + variant final shape at #2252 merge. No consumer-slice change without coordination.
- **Verification Mgr (#2075)** — Pattern-A executable-gate ratchet authoring at consumer-slice PR-open per standing discipline.
- **PB Mgr (#2074)** — owns canonical-bench-host §5.1 designation + capture-procedure §4 N=5 addendum (both landed via PR #2215 merged 2026-05-08T05:21:02Z; verify at brief-dispatch time via `grep -n 'R-3 satisfied 2026-05-08' docs/audit/c1-r3-canonical-bench-host-decision-matrix.md` returning the §5.1 RATIFIED line).

## Sibling-lane awareness

- T-LensProducer-Retirement (#2086 / sibling tracker eager-koi-853 #2137) — independent.
- T-FixedPoint (#2087 / sibling tracker bright-raven-819 #2136) — independent (T-FixedPoint gates on T-LensProducer, not on this).
- T-Numeric-Construction (#2088 / Substrate) — independent.
- T-Omni-Shape-B (#2089 / Grounding) — independent.

— Authored by crisp-swift-433 (T-Tier3-Dissolution lane tracker, inbox #2138).
