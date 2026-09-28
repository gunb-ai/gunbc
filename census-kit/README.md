# Sample census kit (bright-boar-848, used for #12383 and #12436)

Not for merging. It pairs main and head per `.dag` path at two pinned SHAs, and for each path records
(a) the native file refusal (`native_test_context_from_ingest` `file_refusals`) and (b)
`reference_conservation_census_for_path` (the module summary plus every absent atom).

- `refusal_census.dag`: the harness entry (`refusal_census_for_paths`). One ingest per path.
  It prints `ROW` / `REFUSAL` / `CONS` / `ABSENT` lines. **Known defect:** its read step skips an
  unreadable path and prints `ROW` (fail-open). The analyser only pairs a path when BOTH sides
  printed a `CONS` summary, and the conservation half reports such a path as `unreadable`, so this
  does not reach results. Do not trust `ROW` alone.
- `remote_census.sh`: the BuildBuddy dispatch body. It fetches both pinned SHAs and verifies them,
  builds `gunbc` once at head, and binds a **child cgroup `memory.max` per batch, after the build**
  (no `GUNBC_MEMORY_BUDGET_BYTES`, which is the refusal-bypass). It runs each batch at both sides and
  streams rows per batch.
- `wave.sh <wave> <files-per-shard> <batch-size> <cap-bytes> <parallel> [max-shards]`: one wave of
  concurrent dispatches over `remaining.txt`. Runner property: `EstimatedMemory=48GB`. The
  **BuildBuddy free tier kills any run at 1h**, so size the shards to finish inside it.
- `drive.sh`: loops waves until `remaining.txt` stops shrinking. It uses singleton batches (9 files
  per shard, 1 per batch, 21 GiB cap, 2 parallel). For the 315-path sample, one or two waves pair
  about 285; the rest time out even alone.
- `recompute.py`: rewrites `remaining.txt` from completed waves only. `analyze.py`: pairs completed
  waves only and reports recovered atoms, newly absent atoms, refusal changes and unpaired paths.
  **Never analyse a wave while it is still running.** Its shard files are written mid-flight, and I
  once published 11 phantom drops from reading one.

Setup: `export KIT_DIR=<this dir> CENSUS_BASE=<sha> CENSUS_HEAD=<sha>`. Put the population (one path per
line) in `$KIT_DIR/sample.txt`: for the pinned sample, extract
`reference_conservation_stratified_sample_paths` from `src/v2/compiler/reference_conservation_census.dag`
at the base. Six of its paths no longer exist. Then run `./drive.sh` from the repo root.

**Status:** the scripts are those that produced the #12383 and #12436 receipts, with the pinned SHAs and the embedded harness turned into env/arguments AFTER those runs. This parameterised form has NOT been executed end to end: run a 3-file smoke wave first.
