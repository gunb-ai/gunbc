#!/usr/bin/env bash
# Must-pass v4 batch-executor CI gate — the near-term dogfood where the
# v4.workflow.executor .dag is the BATCHING AUTHORITY for the claim suite.
#
# `claim_executor` evaluates the executor-decided plan
# (batch_runner.dag::bre_claim_batches -> List<List<ClaimRef>>) and runs the
# suite batch-by-batch, claims-in-parallel.
#
# Model-first dissolution of the former inline-shell perturb scaffold (the awk
# roster projection + the three inline-python perturbs are gone). The gate now
# splits along the model-vs-orchestration line:
#
#   green:       the plan yields 2 batches and every claim passes (exit 0). The
#                executor places the gating claim in batch 1 and the two derived
#                claims in batch 2 (a later readiness layer).
#
#   --perturb-check adds two discriminating receipts:
#
#     [authority + degenerate] = MODEL semantics, run through ci-claim-gate's
#       uniform witness-body->false perturb over batch_runner.dag's
#       bre_authority_rows_tsv: the dependency input DRIVES the batch count
#       (empty deps -> 1 batch, full deps -> 2, empty suite -> 0). The batching is
#       computed by the .dag (parallel_executor_plan_from_dependencies); these are
#       pure .dag witnesses, not a host re-resolve. Replaces the old inline-python
#       "empty the dep list and re-run the host" + "empty the suite" perturbs.
#
#     [run-loop walk] = CI-ORCHESTRATION behavior, host-resident in claim_executor
#       (--perturb-check): planting the batch-1 gating witness->false must make the
#       run fail closed AND halt the walk before batch 2. Tests ONLY the run-loop
#       walk-halt; the batch structure/ordering it walks is the .dag's (asserted by
#       the model witnesses above). Replaces the old inline-python gating perturb.
#
#     [degenerate host] = CI-ORCHESTRATION fail-closed, exercised through a real
#       consumer: claim_executor is driven with a deliberately empty plan
#       (bre_degenerate_empty_plan -> []) and must exit non-zero — an empty run is
#       never a successful run. Replaces the old inline-python "empty the suite and
#       re-run the host" perturb; the model side (empty in -> zero out) is the
#       bre_empty_suite_yields_no_batches witness above.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

exe="${CLAIM_EXECUTOR:-target/release/claim_executor}"
gate="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
plan_entry="src/v4/test/claim/workflow/batch_runner.dag"
perturb=0

case "${1:-}" in
  --perturb-check) perturb=1 ;;
  "") ;;
  *) echo "usage: $0 [--perturb-check]" >&2; exit 2 ;;
esac

if [[ ! -x "$exe" ]]; then
  echo "error: claim_executor not found at $exe (build: cargo build -p v2-compiler --release)" >&2
  exit 2
fi

echo "== green: executor-decided batches, every claim passes =="
out="$("$exe" --source-root src/v4 --plan-entry "$plan_entry" 2>&1)"
echo "$out"
if ! grep -q "executor plan = 2 batch(es)" <<<"$out"; then
  echo "FAIL: expected 2 executor-decided batches" >&2
  exit 1
fi
if grep -q '^FAIL' <<<"$out"; then
  echo "FAIL: a claim failed in the green pass" >&2
  exit 1
fi
echo "green OK"

if [[ "$perturb" -eq 0 ]]; then
  exit 0
fi

echo "== perturb/authority + degenerate (MODEL): deps drive batch count, via ci-claim-gate =="
if [[ ! -x "$gate" ]]; then
  echo "error: ci-claim-gate not found at $gate (build: cargo build -p ci_claim_gate --release)" >&2
  exit 2
fi
"$gate" \
  --source-root src/v4 \
  --gate-entry "$plan_entry" \
  --rows-fn bre_authority_rows_tsv \
  --notice-title "batch-executor authority" \
  --perturb-check
echo "authority OK (model drives batching; each witness red-under-mutation)"

echo "== perturb/walk (ORCHESTRATION): false gating batch-1 -> fail closed AND halt before batch 2 =="
"$exe" --source-root src/v4 --plan-entry "$plan_entry" --perturb-check
echo "walk OK (run-loop halts dependents on a failed gate)"

echo "== perturb/degenerate (ORCHESTRATION): a 0-batch plan must fail closed, not pass on 0 claims =="
# Drive claim_executor with a deliberately EMPTY plan (bre_degenerate_empty_plan -> [])
# and require a non-zero exit. Exercises the host fail-closed guard (an empty run is never
# a successful run) through a real .dag consumer — no temp-tree, no shell scaffold. Replaces
# the old inline-python "empty the suite nodes and re-run" perturb.
if "$exe" --source-root src/v4 --plan-entry "$plan_entry" --plan-function bre_degenerate_empty_plan; then
  echo "FAIL: claim_executor accepted a 0-batch plan (vacuous pass)" >&2
  exit 1
fi
echo "degenerate OK (host fails closed on an empty plan)"

echo "ALL OK"
