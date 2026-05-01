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
# Single invocation; no arguments. Outputs criterion's JSON estimate files
# under target/criterion/<group>/<bench>/new/estimates.json.
cargo bench --bench tier3_mirror_perf -p v3-compiler
```

**Notes:**

- The bench file is the merged `src/v3/compiler/benches/tier3_mirror_perf.rs` (PR #1362). It registers five `bench_function` calls (see §3 mapping) under one Criterion group via `criterion_main!`.
- Criterion's default sample size is 100; do NOT override unless PB Manager explicitly approves a different statistical window. Larger windows reduce p99 variance at the cost of capture time.
- Run on a quiesced host: no concurrent CPU-intensive jobs. The brief's `≤2× median / ≤5× p99` thresholds assume hardware-stable measurement.
- Capture is one-shot. Once `tier3_baseline.json` lands on `main`, the file is read-only data per the brief; recapture requires Director approval.

---

## 3. Mirror-group → bench-name mapping (verified at HEAD `f66334729`)

The four perf-budget claims in `tier3_mirror_dissolution_perf_within_budget` map to bench names from `src/v3/compiler/benches/tier3_mirror_perf.rs`:

| Mirror slice | Per-mirror claim | Criterion bench name(s) (from `bench_function` arg) |
|---|---|---|
| termination | `tier3_termination_mirror_perf_within_budget` | `tier3_termination_merge_evidence` |
| computation | `tier3_computation_mirror_perf_within_budget` | `tier3_computation_positive_descent_count` + `tier3_computation_lower_same_argument_call` |
| induction | `tier3_induction_mirror_perf_within_budget` | `tier3_induction_type_iteration_dimension_miss` |
| effect-carrier | `tier3_effect_carrier_mirror_perf_within_budget` | `tier3_effects_lane2_linear_read_chain` |

For mirrors with multiple bench names (computation), the per-mirror claim aggregates: median(group) = max(median of contributing benches) and p99(group) = max(p99 of contributing benches). This preserves the "any single bench breaching the bracket fails the gate" semantics of the worker brief §"Composition" `Conj` over per-mirror claims.

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

- `median_ns` and `p99_ns` are non-negative integers in nanoseconds. Round criterion's `point_estimate` from `target/criterion/<bench>/new/estimates.json` to the nearest nanosecond; do not commit decimal/float timings (avoids floating-point platform variance in the JSON itself).
- `host_id` is required and must match the canonical host designated under R-3. CI rejects edits to `tier3_baseline.json` whose `host_id` differs from the recorded canonical host without an accompanying Director approval receipt.
- `git_sha` is the full 40-char SHA, not abbreviated.
- `mirror_groups` keys MUST be exactly the four strings `termination`, `computation`, `induction`, `effect_carrier` — no others, no aliases. The Phase 2 gate uses these as join keys.
- `benches[].name` MUST exactly match a `bench_function` argument in `tier3_mirror_perf.rs` at the captured `git_sha`. CI validation rejects mismatches.

---

## 5. Validation rules (post-capture, pre-commit)

Before the Phase 1 0c PR opens, the operator MUST run the following checks against the produced JSON:

1. **Schema completeness:** every key in §4 is present; no extras; types match.
2. **Bench-name coverage:** for each `git_sha` in `captured_on`, the bench-names listed in `mirror_groups[*].benches[].name` form a *superset of* the bench-names registered in `tier3_mirror_perf.rs` at that SHA. (Subset = missing measurement; superset only allowed if a future bench is intentionally not in the budget — none currently apply.)
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
