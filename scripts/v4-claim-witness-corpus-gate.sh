#!/usr/bin/env bash
# Claim-witness corpus CI gate — uniform `ci-claim-gate` transport, one shard per invocation.
#
# Host-collapse of the prior shell scaffold onto `ci-claim-gate` (the same host probe-selector /
# batch-executor / lens-ci use). Dissolved: the awk/grep roster projection, the inline-python
# per-row perturb, the `gunbc --claim-run` ExpectFail/claim-run classification, the
# GITHUB_RUN_NUMBER spot-perturb rotation, and the row-count guard. The gate model
# (claim_witness_corpus_ci_runner.dag) owns the rows; this script selects the shard's rows-fn +
# source-root(s) and runs ci-claim-gate's GREEN pass plus, on --perturb-check, the per-row
# witness-body->`false` perturb that must go RED.
#
# Shards are ROOT-ALIGNED: ci-claim-gate's --perturb-check copies only the PRIMARY --source-root
# into its temp tree (run_perturb_pass), so every perturbed row must live under the primary root.
#   shard a..e -> primary src/v4 (all rows src/v4-rooted; gate-entry src/v4). Sized ~9-10 rows
#                 each so full per-row --perturb-check (cold-resolve of src/v4 std closures,
#                 ~33s/row measured) stays well under the 20m ceiling at the ×2.6 merge-wave tail.
#   shard f    -> primary dsl    (all rows dsl-rooted). src/v4 is added as a SECOND, read-only
#                 resolve root solely so ci-claim-gate can resolve the gate-entry that holds the
#                 rows-fn (the runner lives under src/v4); no src/v4 row is perturbed in shard f,
#                 so the single-primary-root perturb path is not crossed.
#
# SPOT-PERTURB DROPPED: the prior gate spot-perturbed 1-2 rotating rows/run (GITHUB_RUN_NUMBER)
# because the 20-row monolith full --perturb-check measured 1108s (~18.5m) > 13m. Six smaller
# shards run FULL --perturb-check per shard comfortably under ceiling (per-shard timing receipts
# recorded at sign), so EVERY row is perturb-covered EVERY run — no rotation, no partial coverage.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
shard=""
perturb=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --perturb-check)
      perturb=(--perturb-check)
      shift
      ;;
    --shard)
      shard="$2"
      shift 2
      ;;
    "")
      shift
      ;;
    *)
      echo "usage: $0 --shard a|b|c|d|e|f [--perturb-check]" >&2
      exit 2
      ;;
  esac
done

# Root-aligned: a..e are src/v4-only; f is dsl-primary with src/v4 added read-only for the
# gate-entry (see header). Primary root is listed FIRST (ci-claim-gate perturbs the primary).
case "$shard" in
  a|b|c|d|e) source_root_args=(--source-root src/v4) ;;
  f)         source_root_args=(--source-root dsl --source-root src/v4) ;;
  *)
    echo "error: --shard a|b|c|d|e|f is required" >&2
    exit 2
    ;;
esac

if [[ ! -x "$bin" ]]; then
  echo "error: ci-claim-gate not found at $bin (build: cargo build -p ci_claim_gate --release)" >&2
  exit 2
fi

exec "$bin" \
  "${source_root_args[@]}" \
  --gate-entry src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag \
  --rows-fn "claim_witness_corpus_shard_${shard}_rows_tsv" \
  --notice-title "claim witness corpus (shard ${shard})" \
  "${perturb[@]}"
