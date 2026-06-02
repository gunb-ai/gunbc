# Affected-set CI kill-criterion instrumentation (Wave-1 §11.7)

Work item: `adhoc-0ae3efe5-52b` · dispatched by CI Manager (silent-crane-669) 2026-06-02.

## Operator question this answers (the "kill criterion")

> On what fraction of real PRs does the affected-set actually let us skip expensive work
> (especially the `ci_floor` v2→v4 bootstrap path), and how many wall-clock minutes does that
> save versus a full run?

There is **no pre-coded threshold in-tree**. The deliverable is per-PR measurement data the
operator aggregates to decide whether to flip `ci_floor` from "always run" (its state today) to
affected-set-gated. This is ops analytics, not policy enforcement.

## What it retires (P5 mechanism-b)

**Today's false-positive / unknown:** there is *no aggregate skip-rate data*. `ci_floor` runs
unconditionally on every non-draft PR, and we have no per-PR record of whether the affected-set
*would* have allowed skipping it, nor any minutes-saved estimate. The decision to gate `ci_floor`
is therefore currently unmeasurable — it would be a guess. This receipt makes it a measurement.

## Mechanism

- `tools/ci_affected_components`:
  - `receipt.rs` — `AffectedSetCiReceipt` struct + pure builders (`component_partition`,
    `bootstrap_required`, `saved_minutes`). JSON `schema_version = 1`.
  - `git_read.rs` — shared `git diff --name-only` host transport (also adopted by the existing
    `detect-ci-affected-components` bin; single source for the diff range + fail-closed read).
  - `bin/emit_affected_set_ci_receipt.rs` — emits the receipt JSON.
- `.github/workflows/ci.yml` `affected` job — one emit step + `actions/upload-artifact`
  (`affected-set-ci-receipt`). Measurement only; gates nothing.

## Field semantics

- `selected_components` / `skipped_components` — partition of the six bankruptcy buckets
  (v2, v3, v4, testclaim_corpus, workflow_policy, release_distribution).
- `bootstrap_required` — `v2 || v4` selected ⇒ the `ci_floor` v2→v4 bootstrap path (the dominant
  ~9m bootstrap + ~13m M1 emit-probe cost) would be needed. This is the prediction the skip
  decision turns on.
- `saved_minutes = max(0, estimated_full_run_minutes − actual_run_minutes)`, clamped to `0` when the
  baseline is unset so a missing baseline never reports phantom savings.
- `fail_closed` — `git diff` read failed ⇒ fail-closed superset (all buckets). Skip-rate
  aggregation must **exclude** these rows.

## v1 limitations (explicit, not silent)

- `estimated_full_run_minutes` defaults to **unset (0.0)**. The operator supplies the real
  `ci_floor` p50 baseline (ping silent-crane-669). We deliberately do not invent a constant — an
  overstated baseline would manufacture phantom savings in the aggregate.
- `wall_clock_by_job` / `actual_run_minutes` are emitted empty/zero in v1. The `affected` job runs
  early, before job timings exist; populating them needs a follow-up final-aggregator job that reads
  GHA job timings via the API. `--job-timings <json>` and `--actual-run-minutes` flags already exist
  on the bin for that wiring.
- `selected_claim_count` is `0` in v1 — node-frontier claim selection stays shadow in the Wave-3
  modeled `CiSelectionReceipt`; this ops path does not duplicate it.

## Relationship to PR #4224 (Wave-3 `CiSelectionReceipt`)

Distinct lanes, no duplicate carrier:

- **#4224** = selection-receipt **substrate** (`src/v4/workflow/ci.dag`): modeled `CiSelectionReceipt`,
  Shadow/Active mode, modeled-vs-host parity witness, path to TestClaim projection / future Active
  scheduling.
- **This work** = Wave-1 ops **instrumentation**: skip-rate + minutes-saved analytics. No
  Shadow|Active, no new claim projection, **no amendment to `CiSelectionReceipt`**. The receipt is
  emitted *alongside* the shadow receipt, never inside it. Shared input is the affected-set partition
  over `git diff`; if #4224 lands first they may share the `git_read` transport in
  `ci_affected_components`.
