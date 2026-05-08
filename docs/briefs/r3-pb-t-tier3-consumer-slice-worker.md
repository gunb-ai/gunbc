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
- Helper `fn perf_baseline_measurement(&self, decl_id: DeclarationId, role: &str) -> Result<PerfMeasurement, PerfMeasurementResolveError>`: structural resolve of `{ median_ns, p99_ns }` from declaration's data record fields. Define a small typed `PerfMeasurementResolveError` carrier (variants for `MissingDeclaration`, `WrongConnective`, `MissingField { field }`, `WrongFieldKind { field }`) co-located with the helper. The outer `eval_perf_within_baseline` boundary converts the typed error into `ClaimResult::Fail(format!(...))` with the role label preserved. Per `CODING.md` typed-error discipline; raw `String` is reserved for the `ClaimResult` boundary, not the inner helper.

**Note on jolly-dove-416 prior art:** child session's stripped-from-#2252 `eval_perf_within_baseline` impl is structurally a fit; reuse-with-attribution is acceptable (option-B carve places this code here, not there). Worker should cross-check the stripped form against this brief's spec — not re-invent wholesale.

**Acceptance:** unit tests in `#[cfg(test)] mod` inside `src/v3/compiler/src/test_runner.rs` covering both budget-evaluation paths and resolver fail-closed paths:

*Budget evaluation* (4 cases): pass-when-under-budget, fail-on-median-over, fail-on-p99-over, fail-on-overflow (saturate-on-overflow → Fail).

*Resolver fail-closed* (one case per `PerfMeasurementResolveError` variant — risky boundary, must demonstrate every typed failure path produces `ClaimResult::Fail` with the role label preserved): `MissingDeclaration` (declaration ID absent from DAG), `WrongConnective` (declaration is not a record), `MissingField { field }` (record lacks `median_ns` or `p99_ns`), `WrongFieldKind { field }` (field present but not Int). Test table form is acceptable (parametric over variant + role).

Per `TESTING.md` unit-vs-integration discipline, this is a narrow crate-internal helper; tests live alongside source. Integration coverage in `tests/integration/test_runner_test.rs` is reserved for the pipeline-as-unit gate-clearing path exercised through deliverable §3 (the `tier3_mirror_dissolution_perf_within_budget` TestClaim end-to-end).

### 2. Bench-harness integration with `tier3_mirror_perf.rs`

**File:** `src/v3/compiler/benches/tier3_mirror_perf.rs` + new capture-and-emit driver.

**Shape:**
- Existing skeleton has 4 criterion benches (termination / computation / induction / effect-carrier) — keep as-is for Phase-1 baseline measurement of hand-Rust mirrors.
- Add **Phase-2 bench** (sibling group `tier3_mirror_phase2`): same 4 benches, but invoking the `.dag`-Evaluator path via `TestRunner` end-to-end on a fixture `.dag` program that exercises each mirror's signature operation through the evaluated std body. Phase-1 baseline measures the Rust path; Phase-2 measures the dissolved-mirror path; the gate compares.
- Add a small CLI under `src/v3/compiler/src/bin/` (or analogous) — `tier3_baseline_capture` — that runs `cargo bench tier3_mirror_phase1 -- --measurement-time` per ratified R-7 procedure (N=5, median-of-medians, max-p99-across-runs), serializes to `tier3_baseline.json` schema (per `docs/audit/c1-tier3-baseline-capture-procedure.md`), and prints the JSON.
- Phase-2 measurement path emits same JSON schema; both consumed by §3 below.

**Single-authority discipline** (per INVARIANTS.md P2 + `docs/modeling-discipline.md` Practice 5):
- **The `.dag data` declarations in §3 are the only committed representation of the gate inputs.** Runtime evaluator reads them; the gate is defined by them; nothing else holds parallel copies.
- `tier3_baseline.json` is **not committed**. The capture CLI emits JSON to stdout (or an ephemeral temp path scoped to the regen invocation). A checked-in regen step in the same `cargo run -p v3-compiler --bin tier3_baseline_capture` invocation pipes the JSON into a `.dag data` writer that updates §3 source. The JSON exists only in transit; the final state of the working tree contains only the `.dag` source change.
- Capture-procedure document `docs/audit/c1-tier3-baseline-capture-procedure.md` may reference the JSON schema as an internal serialization shape between capture and regen; it is not a committed artifact.

**Capture invocation** (per ratified R-7, runs only on `ubicloud-standard-2` per ratified R-3): `cargo run -p v3-compiler --bin tier3_baseline_capture -- --N=5 --apply` runs the bench, captures medians + p99, and rewrites the four `.dag data` literals atomically. Without `--apply` the CLI prints a diff for review. Capture-once-and-commit means: the human runs the CLI, reviews the diff, commits the resulting `.dag` change, and discards the JSON intermediate. Recapture only on Director-approved trigger conditions per `docs/audit/c1-tier3-baseline-capture-procedure.md` §5.1.

**Acceptance:** §3 `.dag data` values land via the regen path. No `tier3_baseline.json` file in the working tree post-capture. CI has no JSON-vs-`.dag` drift-detection gate (no duplicates to police).

**Hand-Rust scaffold accounting** (per P5 dissolution-trigger discipline; INVARIANTS.md TRACKED-vs-UNTRACKED-debt):
- New hand-Rust files under this slice: `src/v3/compiler/src/bin/tier3_baseline_capture.rs` (CLI), Phase-2 bench additions in `src/v3/compiler/benches/tier3_mirror_perf.rs`, helper module for `PerfMeasurement` + `PerfMeasurementResolveError` adjacent to `test_runner.rs`.
- **SG-0 census receipt**: capture before/after `EXPECTED_HAND_AUTHORED_NON_TEST` count in the slice PR description; the count is **expected to ratchet UP** by the line count of the new hand-Rust files (this slice is consumer-side scaffolding, not stage0 hand-Rust dissolution; ratchet is acknowledged-and-tracked, not silenced).
- **Per-surface scaffold-vs-steady-state classification** (every new hand-Rust surface gets a disposition):
  - **`bin/tier3_baseline_capture.rs` CLI** — *scaffold*. Retires when capture-once-and-commit completes for the four mirror baselines (operationally one PR, post per-mirror retirement chain). Trigger: "delete `bin/tier3_baseline_capture.rs` after the four `.dag data` baselines land via `--apply` and recapture is no longer expected before R-4 close."
  - **Phase-1 hand-Rust mirror bench groups** in `tier3_mirror_perf.rs` (`bench_termination_mirror`, `bench_computation_mirror`, `bench_induction_mirror`, `bench_effect_carrier_mirror`) — *scaffold*. Retire when the per-mirror retirement workers land — the hand-Rust mirror functions cease to exist, breaking the bench compile. Trigger: "delete each Phase-1 bench group as its corresponding mirror retirement PR lands; remove the `tier3_mirror_phase1` criterion group when the last mirror retires."
  - **Phase-2 bench group** in `tier3_mirror_perf.rs` (`tier3_mirror_phase2` invoking the `.dag`-Evaluator path) — *steady-state regression guard*. Persists post-T-Tier3-close as the gate's runtime measurement source. No dissolution trigger; this is owned consumer-side regression coverage.
  - **`PerfMeasurement` + `PerfMeasurementResolveError` helper module** adjacent to `test_runner.rs` — *steady-state*. Lives as long as `eval_perf_within_baseline` lives, which is the runtime invariant impl for the `PerfWithinBaseline` predicate; that predicate persists post-T-Tier3-close. No dissolution trigger.
- **Receipt**: each *scaffold* surface above gets a `ROADMAP.md` row (or, if no natural slot, an inline entry in the slice PR description) naming the deletion path + trigger. Worker confirms receipts at PR-open time. Steady-state surfaces are documented in the slice PR description as such (with rationale), so future maintainers don't mis-classify them as undeleted scaffolds.

### 3. `tier3_mirror_dissolution_perf_within_budget` claim authoring

**Files:** new `src/v3/std/tier3_perf_budget.dag` (or extension to existing T-Tier3 module).

**Shape** (per `r3-structure.md` §225 + Director c#4403480005 acceptance gate 6 + `docs/audit/c1-tier3-baseline-capture-procedure.md` §3 per-bench conjunction discipline):

The capture-procedure §3 mandates **per-bench conjunction**, NOT per-mirror aggregation: each contributing bench keeps its own `median_ns` / `p99_ns` row, and the gate checks each bench independently. The computation mirror specifically has **two bench rows** (`tier3_computation_positive_descent_count` + `tier3_computation_lower_same_argument_call`); collapsing them to a single budget authority would let a small-budget bench regress arbitrarily within the larger bench's headroom — fail-closed-violation.

Per §3 the budgeted bench-name set at HEAD is exactly 5: `tier3_termination_merge_evidence`, `tier3_computation_positive_descent_count`, `tier3_computation_lower_same_argument_call`, `tier3_induction_type_iteration_dimension_miss`, `tier3_effects_lane2_linear_read_chain`. The `.dag` data + claim shape mirrors that exactly:

```dag
// Phase-1 baselines — one PerfBaselineMeasurement per budgeted bench (5 total)
data tier3_bench_termination_merge_evidence_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_bench_computation_positive_descent_count_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_bench_computation_lower_same_argument_call_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_bench_induction_type_iteration_dimension_miss_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }
data tier3_bench_effects_lane2_linear_read_chain_baseline: PerfBaselineMeasurement = { median_ns: <captured>, p99_ns: <captured> }

// Phase-2 measurements — one row per budgeted bench (5 total)
data tier3_bench_termination_merge_evidence_phase2: PerfBaselineMeasurement = { median_ns: <Phase-2-captured>, p99_ns: <Phase-2-captured> }
// ... four more, one per budgeted bench

// Per-mirror claims — conjunction over the mirror's contributing benches.
// Termination / induction / effect-carrier each cover ONE bench → trivial 1-element conj.
// Computation covers TWO benches → 2-element conj enforcing per-bench fail-closed.
data tier3_termination_mirror_perf_within_budget: TestClaim = TestClaim {
  predicate: PerfWithinBaseline {
    subject: DeclarationRef("tier3_bench_termination_merge_evidence_phase2"),
    comparator: Le,
    baseline_ref: DeclarationRef("tier3_bench_termination_merge_evidence_baseline"),
  },
}
data tier3_computation_mirror_perf_within_budget: TestClaim = TestClaim {
  predicate: Conj {  // both per-bench checks must pass
    PerfWithinBaseline { subject: ..._positive_descent_count_phase2, comparator: Le, baseline_ref: ..._positive_descent_count_baseline },
    PerfWithinBaseline { subject: ..._lower_same_argument_call_phase2, comparator: Le, baseline_ref: ..._lower_same_argument_call_baseline },
  },
}
// ... induction + effect_carrier each 1-element conj over their single bench

// Suite-level: conjunction of the 4 per-mirror claims
data tier3_mirror_dissolution_perf_within_budget: TestClaim = Conj {
  tier3_termination_mirror_perf_within_budget,
  tier3_computation_mirror_perf_within_budget,
  tier3_induction_mirror_perf_within_budget,
  tier3_effect_carrier_mirror_perf_within_budget,
}
```

**Composition note:** per §225 ("≤2× median, ≤5× p99 thresholds"), the runtime invariant impl in §1 applies the ratio at predicate-evaluation time. The `.dag` predicate declares `comparator: Le`; runtime applies the budget multiplier per axis. Suite gate is `Conj` of 4 per-mirror claims; per-mirror claims are `Conj` of 1-or-2 per-bench `PerfWithinBaseline` predicates per the §3 budgeted bench-name set. **No group-level numeric aggregation** at any layer (per capture-procedure §3 fail-closed semantics).

**§3 bench-set drift gate**: a CI assertion verifies the set of `tier3_bench_*_baseline` declaration names equals exactly the §3 budgeted bench-name set; any drift (missing or extra bench) fails CI. Receipts exist in capture-procedure §3 for intentional allowlist exclusions; default is exact-equality.

**Acceptance:** `cargo test -p v3-compiler tier3_mirror_dissolution_perf_within_budget` passes against captured baselines + Phase-2 measurements. Claim is registered in §1.8 ledger as **CONSUMER_LANDED**.

### 4. Cementing receipt: end-to-end T-Tier3 R-4 gate clearing

**Shape:**
- After mirror-dissolution PRs land (the 4 per-mirror retirement workers in PB canvas, separate dispatch chain): re-run Phase-2 capture. Gate clears if all 4 mirror Phase-2 measurements satisfy `≤ baseline × {2, 5}`. Per PB Mgr disposition at PR #2254 review, this measurement run is a separate trigger post-#2204-consumer slice merge (deliverables 1-3 author against mock-shape; deliverable 4 measurement waits for per-mirror retirement chain).
- Receipt artifact: `docs/audit/c1-tier3-perf-budget-receipt.md` — **audit metadata only**: lists `.dag data` declaration names + the commit SHA at which each was captured + the gate-clearing pass/fail status per mirror. **Does not duplicate the numeric values** — the reader follows the cited SHA to read values from the `.dag` source (single-committed-representation discipline). No CI string-equality check is needed because there is no second copy to keep aligned. Carries explicit "execute when per-mirror dispatch completes" note tying the measurement run to retirement-chain landing.
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
