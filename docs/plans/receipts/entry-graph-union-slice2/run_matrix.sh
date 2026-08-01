#!/usr/bin/env bash
# Slice-2 measurement matrix (operator 2026-08-01). Uses snapshotted binary only.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
BIN="$ROOT/target/slice2-measurement-bin/measure_repeated_typecheck_attribution"
OUT="$ROOT/docs/plans/receipts/entry-graph-union-slice2"
COMMON=(--source-root dag --source-root src/v2
  --scan-dir dag/test/claim
  --scan-dir src/v2/test/claim/manual
  --scan-dir src/v2/test/claim/emit)

log_governor() {
  local tag=$1
  local cgroup_limit
  cgroup_limit=$(cat /sys/fs/cgroup/memory.max 2>/dev/null || cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>/dev/null || echo unreadable)
  {
    echo "=== governor/host envelope ($tag) ==="
    echo "cgroup_memory_max_bytes=$cgroup_limit"
    echo "GUNBC_MEMORY_BUDGET_BYTES=${GUNBC_MEMORY_BUDGET_BYTES:-unset}"
    free -b | head -2
    echo "probe_note=serial single-index resolve; adaptive MemoryGovernor not engaged"
  } >>"$OUT/matrix-governor.log"
}

run_one() {
  local name=$1
  shift
  echo "[matrix] starting $name at $(date -Is)" | tee -a "$OUT/matrix-run.log"
  log_governor "$name"
  "$BIN" "${COMMON[@]}" "$@" \
    --receipt-out "$OUT/receipt-${name}.json" \
    2>&1 | tee -a "$OUT/log-${name}.txt"
  echo "[matrix] finished $name at $(date -Is)" | tee -a "$OUT/matrix-run.log"
}

cd "$ROOT"

EXPLICIT=(
  dag/test/claim/ci_floor_measurement_test.dag
  dag/gunbc/output_policy.dag
  dag/test/claim/anomaly_evidence_witness_test.dag
  dag/test/claim/bmc_fan_converge_witness_test.dag
  dag/test/claim/cache_key_completeness_test.dag
  dag/test/claim/ci_render_slowest_hot_style_test.dag
  dag/test/claim/bmc_redfish_grounding_witness_test.dag
)

EXPLICIT_REV=()
for ((i=${#EXPLICIT[@]}-1; i>=0; i--)); do EXPLICIT_REV+=("${EXPLICIT[i]}"); done

# Per-base disjoint 50-entry windows (0-indexed): early / mid / late of each subject roster.
DISJOINT_WINDOW=50

case "${1:-disjoint}" in
  explicit-a)
    run_one explicit-order-a \
      --entry "${EXPLICIT[0]}" --entry "${EXPLICIT[1]}" --entry "${EXPLICIT[2]}" \
      --entry "${EXPLICIT[3]}" --entry "${EXPLICIT[4]}" --entry "${EXPLICIT[5]}" \
      --entry "${EXPLICIT[6]}"
    ;;
  explicit-b)
    run_one explicit-order-b \
      --entry "${EXPLICIT_REV[0]}" --entry "${EXPLICIT_REV[1]}" --entry "${EXPLICIT_REV[2]}" \
      --entry "${EXPLICIT_REV[3]}" --entry "${EXPLICIT_REV[4]}" --entry "${EXPLICIT_REV[5]}" \
      --entry "${EXPLICIT_REV[6]}"
    ;;
  disjoint)
    GUNBC_CI_DIFF_BASE=e30621111f37 run_one narrow-disjoint \
      --entry-offset 0 --max-entries "$DISJOINT_WINDOW"
    GUNBC_CI_DIFF_BASE=b01cdf4d8914 run_one typical-disjoint \
      --entry-offset 117 --max-entries "$DISJOINT_WINDOW"
    GUNBC_CI_DIFF_BASE=0d6ffc4db975 run_one broad-disjoint \
      --entry-offset 235 --max-entries "$DISJOINT_WINDOW"
    ;;
  *)
    echo "usage: $0 [explicit-a|explicit-b|disjoint]" >&2
    exit 2
    ;;
esac
