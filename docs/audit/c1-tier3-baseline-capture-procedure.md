# C1 — Phase 1 0c Baseline Capture Procedure & Schema

**Status:** PROPOSAL (audit/procedure only). Authored 2026-05-01 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Parent:** `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (PR #1331), `docs/audit/c1-tier3-perf-budget-readiness-matrix.md` (PR #1358), `src/v3/compiler/benches/tier3_mirror_perf.rs` (PR #1362, merged).
**Authority basis:** worker brief §"Phase 1 deliverables" item 0c + §"Dependencies" item 5 + readiness matrix §1 row R-3.
**Scope:** docs-only procedure + schema specification for the eventual Phase 1 0c `tier3_baseline.json` capture. **No baseline JSON committed. No fake or local-machine timing data. No code. No CI wiring. No Phase 2 work.**

This artifact captures everything that *can* be specified without a canonical bench host being designated; the actual capture is gated on R-3 (PB Manager canonical CI machine) per the readiness matrix STOP-A.

---

## 1. STOP condition for capture (R-3 not met)

Per `docs/audit/c1-tier3-perf-budget-readiness-matrix.md` §1:

| Prerequisite | State at HEAD `f66334729` | Authority needed |
|---|---|---|
| R-3 — canonical CI machine designated for baseline capture | **NOT MET** — no signal in `docs/r3-structure.md`, `docs/briefs/r3-pb-tier3-perf-budget-worker.md`, or recent CI changes naming the bench host. | PB Manager designate; Substrate Manager if CI-infra cross-cut. |

Worker brief §"Discipline / baseline noise concerns" (line 108) is explicit: *"Phase 1 capture and Phase 2 measurement should run on the same canonical CI machine to keep comparison meaningful (hardware-stable). If CI infrastructure varies, escalate to Substrate Mgr."*

**Per dispatch constraint:** "No fake or local-machine baseline JSON unless the canonical-host authority is explicitly satisfied." This procedure document is therefore the bounded preparatory artifact — schema + commands + validation specification authored once, executed once, by the operator who owns the canonical host.

**Concrete next-unblock shape:** PB Manager authors a one-line addition to `docs/r3-structure.md` (or a sibling brief) of the form:

> *Canonical bench host for C1 Phase 1 0c: `<host-identifier>` (e.g., the CI runner labeled `bench-canonical-1`, or a named workspace machine). Phase 1 0c capture and all Phase 2 perf measurements MUST run on this host; deviation requires Director sign-off and a re-capture of the baseline.*

Once that line lands, this procedure is executable as written.

---

## 2. Capture command

```sh
# Run from workspace root on the canonical bench host.
# Single invocation; no arguments. Outputs criterion's JSON files under
# target/criterion/<bench>/new/ : `estimates.json` (point estimates +
# confidence intervals for mean / median / std_dev / median_abs_dev /
# slope) and `sample.json` (the raw sample iteration counts +
# per-iteration nanosecond totals from this run).
cargo bench --bench tier3_mirror_perf -p v3-compiler
```

**Notes:**

- The bench file is the merged `src/v3/compiler/benches/tier3_mirror_perf.rs` (PR #1362). It registers five `bench_function` calls (see §3 mapping) under one Criterion group via `criterion_main!`.
- Criterion's default sample size is 100; do NOT override unless PB Manager explicitly approves a different statistical window. Larger windows reduce p99 variance at the cost of capture time.
- Run on a quiesced host: no concurrent CPU-intensive jobs. The brief's `≤2× median / ≤5× p99` thresholds assume hardware-stable measurement.
- Capture is one-shot. Once `tier3_baseline.json` lands on `main`, the file is read-only data per the brief; recapture requires Director approval.

**Multi-run discipline (R-7 ratified 2026-05-08 per PB Manager warm-dove-618; Director ratification at gunbc#828 c#4403509523):**

- **N=5 runs** per matrix §4 preferred discipline (N≥3 minimum). Each run is an independent CI invocation of the capture command above.
- `median_ns` in `tier3_baseline.json` = **median of the N per-run medians** (median-of-medians).
- `p99_ns` = **max p99 across the N runs** (conservative pin; absorbs run-to-run tail variance).
- Per-run medians and p99s are computed via the path-(a) extraction helper at §2.1 from each run's `target/criterion/<bench>/new/sample.json`.
- The capture PR commits the extraction helper + per-run intermediate JSONs (or a single combined file recording the N samples) alongside the final `tier3_baseline.json` so the multi-run aggregation is reproducible / auditable.

### 2.1 p99 source — explicit

**Criterion 0.5's `estimates.json` does NOT carry a p99 field.** Its `Estimates` shape names `mean`, `median`, `median_abs_dev`, `slope`, and `std_dev` (point estimate + confidence interval each). p99 must be computed from the raw per-iteration timings. Two paths, **the Phase 1 0c capture PR MUST land one of them — not "p99 is read from estimates.json"**:

- **Path (a) — small extraction helper alongside the capture PR.** Add a tiny script (Python preferred to avoid adding Rust deps; ~30 lines) that reads `target/criterion/<bench>/new/sample.json`, normalizes per-iteration time as `times[i] / iters[i]` (Criterion's sample shape stores iteration counts and total ns per sample slot), sorts the per-iteration ns values, and reports `median_ns` and `p99_ns` (linear interpolation between order-statistic ranks; sample size 100 → p99 ≈ value at rank 99 / 100, or `quantile(0.99)`). Helper output is the canonical input to `tier3_baseline.json`'s `mirror_groups[*].benches[*]` rows. Helper is committed alongside the JSON in the same PR; future rebases of the bench surface re-run it.
- **Path (b) — derive p99 from `estimates.json`'s mean/std_dev under a documented Gaussian assumption (NOT recommended).** `p99 ≈ mean + 2.326 × std_dev`. This is fast but lossy — Criterion's distributions are not guaranteed Gaussian (especially with allocator/cache jitter), and the worker-brief budget bracket exists precisely to absorb tail variance. Path (b) is recorded here only because it might be tempting; **prefer (a)**.

The Phase 1 0c capture PR MUST cite which path it took in its description and (for path (a)) commit the extraction script.

---

## 3. Mirror-group → bench-name mapping (verified at HEAD `f66334729`)

The four perf-budget claims in `tier3_mirror_dissolution_perf_within_budget` map to bench names from `src/v3/compiler/benches/tier3_mirror_perf.rs`:

| Mirror slice | Per-mirror claim | Criterion bench name(s) (from `bench_function` arg) |
|---|---|---|
| termination | `tier3_termination_mirror_perf_within_budget` | `tier3_termination_merge_evidence` |
| computation | `tier3_computation_mirror_perf_within_budget` | `tier3_computation_positive_descent_count` + `tier3_computation_lower_same_argument_call` |
| induction | `tier3_induction_mirror_perf_within_budget` | `tier3_induction_type_iteration_dimension_miss` |
| effect-carrier | `tier3_effect_carrier_mirror_perf_within_budget` | `tier3_effects_lane2_linear_read_chain` |

For mirrors with multiple bench names (computation), the per-mirror claim is the **conjunction over per-bench checks**, NOT a max-aggregated single value. Each contributing bench keeps its own `median_ns` / `p99_ns` baseline row in §4's `mirror_groups[*].benches[]`, and the Phase 2 gate checks each bench independently against its own row: `measured_median_i ≤ 2 × baseline_median_i` AND `measured_p99_i ≤ 5 × baseline_p99_i`, for every contributing bench `i`. The per-mirror claim passes only if **every** contributing bench's per-bench check passes. This is the only fail-closed semantics: aggregating to a group-level max baseline would let a small-budget bench regress arbitrarily within the group max's headroom while the group-level ratio still appears under bracket. Worker brief §"Composition" `Conj` over per-mirror claims composes downward into `Conj` over per-bench checks; there is no group-level numeric aggregation in the gate path.

---

## 4. `tier3_baseline.json` schema

The committed baseline file lives at `src/v3/compiler/benches/tier3_baseline.json`. JSON Schema-style spec:

```json
{
  "$schema": "C1 Phase 1 baseline format v1",
  "captured_on": {
    "host_id": "<canonical-host-identifier>",
    "git_sha": "<full sha of HEAD at capture time>",
    "criterion_version": "0.5",
    "rustc_version": "<output of `rustc --version`>",
    "captured_at": "<RFC 3339 UTC timestamp>"
  },
  "mirror_groups": {
    "termination": {
      "claim": "tier3_termination_mirror_perf_within_budget",
      "benches": [
        {
          "name": "tier3_termination_merge_evidence",
          "median_ns": <integer>,
          "p99_ns": <integer>
        }
      ]
    },
    "computation": {
      "claim": "tier3_computation_mirror_perf_within_budget",
      "benches": [
        { "name": "tier3_computation_positive_descent_count", "median_ns": <integer>, "p99_ns": <integer> },
        { "name": "tier3_computation_lower_same_argument_call", "median_ns": <integer>, "p99_ns": <integer> }
      ]
    },
    "induction": {
      "claim": "tier3_induction_mirror_perf_within_budget",
      "benches": [
        { "name": "tier3_induction_type_iteration_dimension_miss", "median_ns": <integer>, "p99_ns": <integer> }
      ]
    },
    "effect_carrier": {
      "claim": "tier3_effect_carrier_mirror_perf_within_budget",
      "benches": [
        { "name": "tier3_effects_lane2_linear_read_chain", "median_ns": <integer>, "p99_ns": <integer> }
      ]
    }
  }
}
```

**Field rules:**

- `median_ns` and `p99_ns` are **strictly positive integers** in nanoseconds (> 0; matches §5 rule 4). `median_ns` is the `point_estimate` for the median field of `target/criterion/<bench>/new/estimates.json`, rounded to the nearest nanosecond. `p99_ns` is **derived per §2.1** from `target/criterion/<bench>/new/sample.json` raw per-iteration timings (Criterion 0.5's `estimates.json` has no p99 field); use path (a) extraction helper or path (b) Gaussian approximation as documented there. Do not commit decimal/float timings — round both fields to integer nanoseconds before writing the JSON to avoid floating-point platform variance.
- `host_id` is required and must match the canonical host designated under R-3. CI rejects edits to `tier3_baseline.json` whose `host_id` differs from the recorded canonical host without an accompanying Director approval receipt.
- `git_sha` is the full 40-char SHA, not abbreviated.
- `mirror_groups` keys MUST be exactly the four strings `termination`, `computation`, `induction`, `effect_carrier` — no others, no aliases. The Phase 2 gate uses these as join keys.
- `benches[].name` MUST exactly match a `bench_function` argument in `tier3_mirror_perf.rs` at the captured `git_sha`. CI validation rejects mismatches.

---

## 5. Validation rules (post-capture, pre-commit)

Before the Phase 1 0c PR opens, the operator MUST run the following checks against the produced JSON:

1. **Schema completeness:** every key in §4 is present; no extras; types match.
2. **Bench-name exact match (fail-closed):** the set of bench-names listed in `mirror_groups[*].benches[].name` MUST equal exactly the budgeted bench-name set defined in §3 — no missing names (every budgeted bench measured), no extra names (no out-of-budget bench in the JSON). Concretely at HEAD `f66334729` the required set is exactly `{tier3_termination_merge_evidence, tier3_computation_positive_descent_count, tier3_computation_lower_same_argument_call, tier3_induction_type_iteration_dimension_miss, tier3_effects_lane2_linear_read_chain}` (5 names). If a future bench is intentionally added to `tier3_mirror_perf.rs` but excluded from the budget, that exclusion MUST be a separate explicit allowlist row in §3 of this procedure document (with a receipt) and the JSON's bench-name set continues to equal the §3-budgeted set, not all bench-names registered in the bench file. Supersets are NOT permitted; subsets are NOT permitted.
3. **Sanity bands:** for each bench, `p99_ns >= median_ns` (criterion guarantees this; reject the JSON if violated as it indicates capture corruption).
4. **Non-zero:** every `median_ns` and `p99_ns` is `> 0`. A zero indicates measurement failure (criterion below floor), not a real timing.
5. **Host stability:** `captured_on.host_id` matches the R-3 canonical host string; reject if not.
6. **Sample size receipt:** the operator's PR description includes the criterion sample size used (default 100) and the effective measurement budget (criterion's reported `target_time` × `sample_size`). If non-default, justify in the PR body.

These rules are spec; their enforcement is a Phase 2 capture-side concern (the Phase 2 PR adds a reader + the structural-acceptance predicate per worker brief §"Acceptance gate" path (a) `PerfWithinBaseline` or path (b) `ExecuteCommand`).

---

## 6. What this PR explicitly does NOT do

- ❌ Commit any `tier3_baseline.json` (real or placeholder) to the repo. Per dispatch constraint and worker brief STOP condition: capture is gated on R-3.
- ❌ Add a `criterion_main!` change, alter `tier3_mirror_perf.rs`, or extend the bench surface. Bench file is single-authority for the measurement targets; this doc only specifies how to read its output.
- ❌ Add CI wiring. Phase 1 capture is one-shot; no repeating CI job is appropriate until Phase 2's `PerfWithinBaseline` predicate is authored.
- ❌ Author the `PerfWithinBaseline` substrate predicate. That is a Substrate Manager call (worker brief §"Acceptance gate"), gated separately.
- ❌ Capture timing data on a non-canonical machine and commit it as `tier3_baseline.json`. Worker brief §"Discipline" forbids this; dispatch constraint reaffirms it.
- ❌ Add new hand-authored Rust files. (No SG-0 entry needed for a docs-only PR.)

---

## 7. Routing question (single)

**For PB Manager.** The canonical bench host (R-3) needs a one-line designation in `docs/r3-structure.md` or a sibling authority doc. Once that lands, this procedure is executable. Suggested form in §1 above. Substrate Manager involvement only if the choice cross-cuts CI infrastructure (e.g., requires a new dedicated runner).

---

## 8. Acceptance summary

This procedure document is intentionally bounded:

- §1 records the STOP condition (R-3 unmet) and the concrete next-unblock shape.
- §2 specifies the capture command.
- §3 maps mirror groups to bench names, verified against `src/v3/compiler/benches/tier3_mirror_perf.rs` at HEAD `f66334729`.
- §4 specifies the `tier3_baseline.json` schema.
- §5 specifies validation rules.
- §6 enumerates non-goals.
- §7 routes one question to PB Manager.

**No capture has been run. No timing data exists in this PR.** Phase 1 0c remains gated on R-3.
