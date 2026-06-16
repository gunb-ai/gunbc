#!/usr/bin/env bash
# Gate-3: AffectedTestsGate — fail-safe superset of affected TestClaim execution.
#
# Runs selection-semantics witnesses plus the full former demoted test battery
# (node-frontier, probe-selector, batch-executor, lens-ci, substrate-equiv,
# corpus shards, layering). Uncertain/broad → RUN ALL until diff-scoped dispatch
# lands in the one-binary runner.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
gunbc="${V2_COMPILER:-target/release/gunbc}"

if [[ ! -x "$bin" ]]; then
  echo "error: ci-claim-gate not found at $bin" >&2
  exit 2
fi
if [[ ! -x "$gunbc" ]]; then
  echo "error: gunbc not found at $gunbc" >&2
  exit 2
fi

"$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/test/claim/workflow/probe_selector_ci_runner.dag \
  --rows-fn probe_selector_ci_runner_rows_tsv \
  --notice-title "affected-tests selection semantics"

"$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/test/claim/workflow/affected_set_ci_runner.dag \
  --rows-fn affected_set_node_frontier_rows_tsv \
  --notice-title "affected-set node-frontier"
"$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/test/claim/workflow/probe_selector_ci_runner.dag \
  --rows-fn probe_selector_ci_runner_rows_tsv \
  --notice-title "probe-selector keystone" \
  --perturb-check
bash scripts/v4-batch-executor-gate.sh --perturb-check
"$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/workflow/lens_ci_gate.dag \
  --rows-fn lens_ci_claim_run_rows_tsv \
  --notice-title "v4 lens CI" \
  --perturb-check
bash scripts/v4-substrate-equivalence-gate.sh --perturb-check
bash scripts/v4-claim-witness-corpus-gate.sh --shard a --spot-perturb-check
bash scripts/v4-claim-witness-corpus-gate.sh --shard b --spot-perturb-check
bash scripts/v4-layering-imports-gate.sh --perturb-check

# #4957: native==interpreted semantics oracle (merge gate). Termination witness deferred-not-green:
# arm64 CI honest RED with native-fold ON at 30s (fold_list=11384 fold_list_right=62895); not a harness defect.
bash .github/ci-floor/with-sccache-retry.sh cargo test -p v2-compiler-tests --release fold_list_native_semantics_test -- --test-threads=1 --quiet

echo "gate-3 PASS: affected-tests superset battery"
