# C1 — Phase 1 0c Baseline Capture Procedure & Schema

**Status:** PROPOSAL + **Phase-1 executable addendum** (R-7 baseline may land on `main`). Original shell 2026-05-01 (silent-boar-29); canonical host **R-3 = `ubicloud-standard-2`** ratified in `docs/audit/c1-r3-canonical-bench-host-decision-matrix.md` (2026-05-08).
**Parent:** `docs/briefs/r3-pb-tier3-perf-budget-worker.md` (PR #1331), `docs/audit/c1-tier3-perf-budget-readiness-matrix.md` (PR #1358), `src/v3/compiler/benches/tier3_mirror_perf.rs` (PR #1362, merged).
**Authority basis:** worker brief §"Phase 1 deliverables" item 0c + §"Dependencies" item 5 + readiness matrix §1 row R-3.
**Scope:** Schema + capture commands + validation for `tier3_baseline.json`. **Phase-1 PRs that freeze R-7 commit** `src/v3/compiler/benches/tier3_baseline.json` alongside honest metadata (§4–§5). **Phase-2** perf predicate / substrate wiring stays out of scope here.

Executable landing forbids **fabricated** provenance (`host_id` must match the machine that executed the benches — **never** mint `ubicloud-standard-2` unless those runs happened on that label). A narrow **bootstrap** path is defined in §4 / §5 when the Ubicloud workflow is not dispatchable yet; it still uses **truthful** `host_id`.

---

## 1. Canonical host R-3 (baseline capture)

Worker brief §"Discipline / baseline noise concerns" (line 108) is explicit: *"Phase 1 capture and Phase 2 measurement should run on the same canonical CI machine to keep comparison meaningful (hardware-stable). If CI infrastructure varies, escalate to Substrate Mgr."*

**R-3 designation (ratified):** **`ubicloud-standard-2`** — see `docs/audit/c1-r3-canonical-bench-host-decision-matrix.md` §5.1 + worker dispatch at gunbc **#828**. **Standard case:** `cargo bench` capture for a committed baseline SHOULD run on that label (via `.github/workflows/tier3-baseline-capture.yml` once the workflow exists on `default`) so `captured_on.host_id` records `ubicloud-standard-2`.

**Bootstrap sequencing exception (single authority, P2):** the *first* committed `tier3_baseline.json` may land in the **same PR** that introduces the Ubicloud `workflow_dispatch` workflow, before `gh workflow run` is available against `default`. In that narrow case the capture runs on the **actual** operator/session machine, `captured_on.host_id` is **that honest identifier** (not `ubicloud-standard-2`), and the Phase-1 PR MUST reconcile provenance in its description (**no retroactive `host_id` relabel**).

This exception **does not** relax the worker brief §"Discipline" **same-machine comparison** discipline for substantive perf gates—it only unlocks early **presence** of a JSON artifact and the forwarded workflow YAML. **`R-7` authority is split:**
- **`R‑7.presence`** (grep / sequencing / STOP triage needing a tracked `tier3_baseline.json` path): satisfied by merging any honest-schema JSON at `src/v3/compiler/benches/tier3_baseline.json`.
- **`R‑7.canonical coherence`** (**mirror dissolution slices + Phase‑2 canonical measurement** pairing `PerfWithinBaseline`/budget claims against timings collected on **`ubicloud-standard-2`**): **NOT satisfied** solely by a non‑Ubicloud bootstrap seed—the committed JSON **MUST either** record `captured_on.host_id == ubicloud-standard-2` captured via `.github/workflows/tier3-baseline-capture.yml` (landed artifact → PR replacing baseline rows/metadata) **or** bear a **Director / PB-published waiver receipt** affirming dissolution may proceed temporarily against bootstrap numbers (fail-closed if neither).

**Operational default:** after this workflow YAML reaches `default`, operators run **`workflow_dispatch`**, copy the **`tier3-baseline-json`** artifact into‑tree (**same PR stack or immediate follow‑up PR**) so `host_id` + medians authoritative for dissolution work match **Ubicloud**, unless Director waives coherence.

**Director / PB** sign-off gates any waiver-class shortcut; speculative “eventually recapture” without an explicit STOP decision is insufficient for **`R‑7.canonical coherence`** clearance.

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
- The capture landing commits **`scripts/aggregate_tier3_baseline.py`** (bundle aggregation + stamping) + the final **`tier3_baseline.json`**. Bundled per-run **`criterion/`** subtrees SHOULD live either **in-repo** alongside the merge (**bootstrap / audited local sequencing**) **or** as **retained Actions artifacts** when capture runs via **`.github/workflows/tier3-baseline-capture.yml`** (preferred once dispatchable on `default`); do not silently drop reproducibility anchors for the **`N`** runs that fed the median-of-medians / max-p99 policy.

### 2.1 p99 source — explicit

**Criterion 0.5's `estimates.json` does NOT carry a p99 field.** Its `Estimates` shape names `mean`, `median`, `median_abs_dev`, `slope`, and `std_dev` (point estimate + confidence interval each). p99 must be computed from the raw per-iteration timings. Two paths, **the Phase 1 0c capture PR MUST land one of them — not "p99 is read from estimates.json"**:

- **Path (a) — small extraction helper alongside the capture PR.** Add a tiny script (Python preferred to avoid adding Rust deps; ~30 lines) that reads `target/criterion/<bench>/new/sample.json`, normalizes per-iteration time as `times[i] / iters[i]` (Criterion's sample shape stores iteration counts and total ns per sample slot), sorts the per-iteration ns values, and reports `median_ns` and `p99_ns` (linear interpolation between order-statistic ranks; sample size 100 → p99 ≈ value at rank 99 / 100, or `quantile(0.99)`). Helper output is the canonical input to `tier3_baseline.json`'s `mirror_groups[*].benches[*]` rows. Helper is committed alongside the JSON in the same PR; future rebases of the bench surface re-run it.
- **Path (b) — derive p99 from `estimates.json`'s mean/std_dev under a documented Gaussian assumption (NOT recommended).** `p99 ≈ mean + 2.326 × std_dev`. This is fast but lossy — Criterion's distributions are not guaranteed Gaussian (especially with allocator/cache jitter), and the worker-brief budget bracket exists precisely to absorb tail variance. Path (b) is recorded here only because it might be tempting; **prefer (a)**.

The Phase 1 0c capture PR MUST cite which path it took in its description and (for path (a)) commit the extraction script.

---

## 3. Mirror-group → bench-name mapping (authoritative names; reconcile `tier3_mirror_perf.rs` at the artifact's `captured_on.git_sha`)

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
- **`host_id` (P2 honesty):** required; MUST identify the machine that executed the benches—**never** spoof `ubicloud-standard-2` unless those runs actually occurred on that label.
  - **Standard commits:** SHOULD use **`ubicloud-standard-2`** (captured via `.github/workflows/tier3-baseline-capture.yml` once dispatchable).
  - **Bootstrap seed landing** (workflow YAML introduced in the same Phase-1 PR—`workflow_dispatch` not yet on `default`): **`host_id` MUST remain the truthful non-Ubicloud identifier**, with PR-body reconciliation naming **forward canonical authority** (`ubicloud-standard-2`) per §1. Replacing that seed via Ubicloud artifact follows **Director / PB-approved recapture** (worker brief) — **not** by silently rewriting `host_id`.
- `git_sha` is the full 40-char SHA, not abbreviated.
- **`captured_at`:** RFC 3339 UTC for when **`tier3_baseline.json`** was **finalized** (**`scripts/aggregate_tier3_baseline.py`** stamps aggregator wall clock—intentional audit receipt); it **need not** match any single **`cargo bench`** end instant. Bench execution spans live in archived **`criterion/`** bundles + workflow/job logs retained with the aggregate PR or artifact lineage.
- `mirror_groups` keys MUST be exactly the four strings `termination`, `computation`, `induction`, `effect_carrier` — no others, no aliases. The Phase 2 gate uses these as join keys.
- `benches[].name` MUST exactly match a `bench_function` argument in `tier3_mirror_perf.rs` at the captured `git_sha`. CI validation rejects mismatches.

---

## 5. Validation rules (post-capture, pre-commit)

Before the Phase 1 0c PR opens, the operator MUST run the following checks against the produced JSON:

1. **Schema completeness:** every key in §4 is present; no extras; types match.
2. **Bench-name exact match (fail-closed):** the distinct names listed in `mirror_groups[*].benches[].name` MUST equal exactly the §3 budgeted set — no missing names (every budgeted bench measured), no extra names (no out-of-budget bench in the JSON). That budgeted five-name set is `{tier3_termination_merge_evidence, tier3_computation_positive_descent_count, tier3_computation_lower_same_argument_call, tier3_induction_type_iteration_dimension_miss, tier3_effects_lane2_linear_read_chain}` (each appearing once across `mirror_groups`, matching `tier3_mirror_perf.rs` at this JSON file's own `captured_on.git_sha`). If a future bench is intentionally added to `tier3_mirror_perf.rs` but excluded from the budget, that exclusion MUST be a separate explicit allowlist row in §3 of this procedure document (with a receipt) and the JSON's bench-name set continues to equal the §3-budgeted set, not all bench-names registered in the bench file. Supersets are NOT permitted; subsets are NOT permitted.
3. **Sanity bands:** for each bench, `p99_ns >= median_ns` (criterion guarantees this; reject the JSON if violated as it indicates capture corruption).
4. **Non-zero:** every `median_ns` and `p99_ns` is `> 0`. A zero indicates measurement failure (criterion below floor), not a real timing.
5. **`host_id` coherence:** Either **`host_id == ubicloud-standard-2`** (standard captures) **or** the **`host_id` matches the bootstrap exception** (§1 / §4) with the Phase-1 PR documenting reconciliation—reject fabricated `ubicloud-standard-2` labels absent Ubicloud-backed runs / dishonest relabeling.
6. **Sample size receipt:** the operator's PR description includes the criterion sample size used (default 100) and the effective measurement budget (criterion's reported `target_time` × `sample_size`). If non-default, justify in the PR body.

These rules are spec; their enforcement is a Phase 2 capture-side concern (the Phase 2 PR adds a reader + the structural-acceptance predicate per worker brief §"Acceptance gate" path (a) `PerfWithinBaseline` or path (b) `ExecuteCommand`).

---

## 6. What this procedure does **not** cover (by design)

- ❌ **Fabricate provenance** for `tier3_baseline.json` (including forging `host_id` or timing rows).
- ❌ **Silent** “local seed” without PR reconciliation when `host_id` is not `ubicloud-standard-2` (bootstrap path requires §1 / §4 honesty + description).
- ❌ Author the **`PerfWithinBaseline`** substrate predicate (Substrate Manager; worker brief §"Acceptance gate").
- ❌ Extend `tier3_mirror_perf.rs` bench surface without updating §3 mapping + JSON row set (fail-closed naming).

**Historical note:** the *original* drafting PR for this document was explicitly **docs-only** (“do not commit baseline JSON yet”). **Executable Phase-1 landing** supersedes that historical non-goal: committing `src/v3/compiler/benches/tier3_baseline.json` **is** required for R-7 once capture discipline is satisfied.

---

## 7. Routing (authority)

- **R-3 host** — ratified as **`ubicloud-standard-2`**; see `docs/audit/c1-r3-canonical-bench-host-decision-matrix.md`.
- **Bootstrap seed vs dissolution coherence** — PB Manager / Director record whether **`R‑7.canonical coherence`** clears via Ubicloud refreshed JSON (**default**) versus a **narrowly scoped waiver**; absence of waiver + non‑Ubicloud `host_id` means dissolution-linked perf benches stay blocked on canonical pairing.

---

## 8. Acceptance summary

- §1 designates R-3 (`ubicloud-standard-2`) + **bootstrap sequencing** exception (`host_id` honest; **split `R‑7.presence` vs `R‑7.canonical coherence`** for dissolution-linked perf parity).
- §2–§3 prescribe capture + bench-name mapping.
- §4–§5 prescribe schema + validation (including **`host_id` honesty**).
- §6 lists out-of-scope substrate / fabrication classes.
- §7 names authority for policy on seed vs refresh.
