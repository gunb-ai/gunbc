#!/usr/bin/env bash
set -euo pipefail
export GIT_PAGER=cat PAGER=cat
(
  while true; do echo "HEARTBEAT $(date -u +%H:%M:%S)"; sleep 60; done
) &
HB=$!
trap 'kill $HB 2>/dev/null || true' EXIT
export PATH="/opt/cargo/bin:${PATH}"
export RUSTFLAGS="-D warnings"
export GUNBC_MEMORY_BUDGET_BYTES=8589934592
export GITHUB_ACTIONS=true

echo "HOST=$(uname -m) HEAD=$(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch
BIN=./target/release

echo "=== WET ==="
set +e
"$BIN/gunbc" run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/auth/gcp_iam_converge_run.dag --function gcp_iam_converge_request_wet
WET=$?
set -e
echo "WET_RC=${WET}"

fns() {
  grep -E '^test fn ' "$1" | sed -E 's/^test fn ([A-Za-z0-9_]+).*/\1/' | paste -sd, -
}

echo "=== CLAIM_BATCH ==="
set +e
"$BIN/claim_batch" --source-root dag --source-root src/v2 \
  --entry dag/test/claim/gcp_iam_converge_witness.dag --functions "$(fns dag/test/claim/gcp_iam_converge_witness.dag)" \
  --entry dag/test/claim/heal_publisher_provision_witness.dag --functions "$(fns dag/test/claim/heal_publisher_provision_witness.dag)" \
  --entry dag/test/claim/auth/branch_run_grant_witness.dag --functions "$(fns dag/test/claim/auth/branch_run_grant_witness.dag)"
CB=$?
set -e
echo "CLAIM_BATCH_RC=${CB}"
echo "SENTINEL_DONE wet=${WET} claim_batch=${CB} head=$(git rev-parse HEAD)"
