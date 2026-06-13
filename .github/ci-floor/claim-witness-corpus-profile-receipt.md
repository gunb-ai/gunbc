# Claim-witness corpus CI profile (profile C) — dag-eval overhead lens

Sign runner cross-check: `scripts/v4-claim-witness-corpus-gate.sh` monolith 1108s (~18.5m);
sharded ~10 rows/shard → ~8.4m uncontended/shard (7.4m Phase A + ~1m spot) per
`.github/workflows/ci.yml` / `claim-witness-corpus-exclusions.md`.

Harness: `scripts/v4-claim-witness-corpus-profile.sh` (same row authority as the gate).

## Overhead lens (per ExpectPass witness)

| Column | Meaning |
|--------|---------|
| `green_batch_s` | Amortized share of `claim_batch` for the witness's entry group (CI Phase A path) |
| `spot_perturb_s` | Temp-copy + perturb + standalone `gunbc run` (CI spot-perturb path) |
| `cold_run_s` | Standalone `gunbc run` without batch amortization (subprocess resolve tax baseline) |
| `batch_savings_s` | `cold_run_s − green_batch_s` (claim_batch win vs per-row subprocess) |
| `overhead_s` | `cold_run_s − spot_perturb_s` (resolve delta between green subprocess and perturb subprocess) |

Shard totals: `green_batch_wall_s` (sum of entry-group batch walls) + ExpectFail cold runs
≈ CI Phase A; `ci_spot_perturb_estimate_s` = 2/N × sum(spot) models rotating 2-row spot check.

## Cold-sample (shard a, full subprocess baseline)

Early sample before `--skip-cold` full-shard run (contended sign runner, CARGO_BUILD_JOBS=2):

| label | cold_run_s | green_batch_s | spot_perturb_s | batch_savings_s |
|-------|-----------:|--------------:|---------------:|----------------:|
| mvp1 emit matches serialize | 149 | 85 | 133 | 64 |
| mvp1 emit ingest round trip | 152 | 85 | 123 | 67 |

**Reading:** claim_batch cuts per-witness wall ~43% vs cold subprocess on shared-entry rows;
spot-perturb is nearly as expensive as cold (~120–130s) because each perturb path pays a full
resolve in a temp tree — the dominant .dag-interpreter tax is resolve+compile, not Bool eval.

## Measured shard a (2026-06-13, `--skip-cold`, contended sign runner)

| metric | seconds | CI sign-runner ref |
|--------|--------:|-------------------|
| `green_batch_wall_s` (Phase A) | **649** (~10.8m) | ~444s (~7.4m) uncontended |
| `sum_spot_perturb_s` (all 14 pass rows) | 1276 | — |
| `ci_spot_perturb_estimate_s` (2/14 rotation) | **182** (~3.0m) | ~60s (~1m) uncontended |
| **estimated CI wall** (Phase A + spot) | **831** (~13.8m) | ~504s (~8.4m) uncontended |

Per-entry Phase A (`green_batch_wall_s` decomposition):

| entry | wall_s | witnesses |
|-------|-------:|----------:|
| mvp1_dag_add_round_trip.dag | 145 | 2 |
| comprep_eval_by_execution.dag | 106 | 1 |
| comprep_branch_eval_by_execution.dag | 77 | 1 |
| bind_demand_driven_eval.dag | 75 | 1 |
| comprep_add_body_emit_typescript.dag | 87 | 4 |
| program_partition_comprep_body_emit.dag | 75 | 3 |
| sg2_type_expression_projection.dag | 84 | 2 |

Spot-perturb per witness spans **63–111s** (median ~92s). Dominant cost is resolve+compile in
temp tree, not Bool eval — same overhead lens as lens_ci / node-frontier profilers.

## Measured shard b (2026-06-13, `--skip-cold`, contended sign runner)

| metric | seconds | notes |
|--------|--------:|-------|
| `green_batch_wall_s` (Phase A) | **535** (~8.9m) | 12 ExpectPass + 1 ExpectFail row |
| `sum_spot_perturb_s` (12 pass rows) | 598 | — |
| `ci_spot_perturb_estimate_s` (2/12 rotation) | **99** (~1.7m) | — |
| ExpectFail cold (`witness_sg2_arrow`) | 61 | baseline row only |
| **estimated CI wall** (Phase A + spot) | **634** (~10.6m) | 535+99 |

## Combined shards (profile C receipt)

| shard | Phase A (green_batch) | spot (2-row est.) | total est. |
|-------|----------------------:|------------------:|-----------:|
| a | 649s (~10.8m) | 182s | 831s (~13.8m) |
| b | 535s (~8.9m) | 99s | 634s (~10.6m) |
| **parallel CI** | max=649s | max=182s | **~831s (~13.8m)** |

Sign-runner uncontended target was ~8.4m/shard; contended measurement is ~1.6× higher on
Phase A. Spot-perturb estimate (~3m shard a, ~1.7m shard b) also exceeds the ~1m sign-runner
receipt — resolve+compile dominates, not Bool eval.

## Full-shard receipts

Generated TSV + summary land under `.ci-profile-corpus/` when the harness is run locally:

```bash
cargo build -p v2-compiler --release --bin gunbc --bin claim_batch
bash scripts/v4-claim-witness-corpus-profile.sh --both --skip-cold --out .ci-profile-corpus
# optional cold baseline on one shard (slow):
bash scripts/v4-claim-witness-corpus-profile.sh --shard a --out .ci-profile-corpus
```

See `corpus_shard_{a,b}_profile.tsv` and `corpus_shard_{a,b}_summary.txt` for per-witness rows.
