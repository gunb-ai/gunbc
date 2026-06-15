#!/usr/bin/env bash
# Gate-3: AffectedTestsGate — fail-safe superset of affected TestClaim execution.
#
# Runs selection-semantics witnesses plus the full former demoted test battery
# (node-frontier, probe-selector, batch-executor, lens-ci, substrate-equiv,
# corpus shards, layering). Also re-homes lightweight execution consumers retired
# from removed workflow jobs: doc_refs (doc-authority), timeout_budgets (ceiling
# ratchet), bootstrap (regen_stage0 --verify, get-off-v3 census). Uncertain/broad
# → RUN ALL until diff-scoped dispatch lands in the one-binary runner.

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

# Former doc_refs job: doc-authority reference gate (diff-scoped vs origin/main).
# ci.yml uses fetch-depth: 1; deepen and materialize origin/main so merge-base exists.
git fetch --no-tags origin "main:refs/remotes/origin/main" --deepen=200
python3 scripts/check_doc_refs.py --changed origin/main
python3 scripts/check_doc_refs.py --check-graph

# Former timeout_budgets job: workflow timeout-minutes ceiling ratchet.
python3 scripts/check_ci_timeout_budgets.py
python3 scripts/check_ci_timeout_budgets.py --perturb-check

# Former ci_floor: stage0 generated-output freshness (BOOTSTRAP.md).
bash .github/ci-floor/with-sccache-retry.sh \
  cargo run --release -p v2-compiler --bin regen_stage0 -- --verify

# Former ci_floor_parity: get-off-v3 caller census ceiling (E-10 consumer; PR #4659).
bash .github/ci-floor/with-sccache-retry.sh \
  cargo test -p v3-compiler --test integration \
  get_off_v3_compile_to_dag_census_test::get_off_v3_compile_to_dag_caller_count_is_at_or_below_ceiling \
  -- --exact --quiet

"$bin" \
  --source-root src/v4 \
  --gate-entry src/v4/test/claim/workflow/probe_selector_ci_runner.dag \
  --rows-fn probe_selector_ci_runner_rows_tsv \
  --notice-title "affected-tests selection semantics"

bash scripts/v4-affected-set-node-frontier-gate.sh --green-only
bash scripts/v4-probe-selector-keystone-gate.sh --perturb-check
bash scripts/v4-batch-executor-gate.sh --perturb-check
bash scripts/v4-lens-ci-gate.sh --perturb-check
bash scripts/v4-substrate-equivalence-gate.sh --perturb-check
bash scripts/v4-claim-witness-corpus-gate.sh --shard a --spot-perturb-check
bash scripts/v4-claim-witness-corpus-gate.sh --shard b --spot-perturb-check
bash scripts/v4-layering-imports-gate.sh --perturb-check

echo "gate-3 PASS: affected-tests superset battery"
