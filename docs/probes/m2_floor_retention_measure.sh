#!/usr/bin/env bash
# Dispatch J — M2-aware floor-retention measurement harness (measurement only).
# Runs claim_executor on the real floor path; captures logs for receipt extraction.
# Usage: docs/probes/m2_floor_retention_measure.sh <probe-name> <plan-function>
# Env: CLAIM_EXECUTOR_BIN (default: worktree target/release/claim_executor-m2-* snapshot)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

SHA="$(git rev-parse HEAD)"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
PROBE="${1:?probe name}"
PLAN_FN="${2:?plan function}"

BIN="${CLAIM_EXECUTOR_BIN:-}"
if [[ -z "$BIN" ]]; then
  BIN="target/release/claim_executor-m2-${SHA:0:7}"
  if [[ ! -x "$BIN" ]]; then
  cp -f target/release/claim_executor "$BIN"
  fi
fi

OUT_DIR="docs/probes/m2_floor_retention_${STAMP}"
mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/${PROBE}.log"
META="$OUT_DIR/${PROBE}.meta"

{
  echo "main_sha=$SHA"
  echo "probe=$PROBE"
  echo "plan_function=$PLAN_FN"
  echo "binary=$BIN"
  echo "started_utc=$STAMP"
  echo "github_event=${GITHUB_EVENT_NAME:-unset}"
  echo "schedule_retention_evict=${GUNBC_SCHEDULE_RETENTION_EVICT:-default-on}"
  echo "floor_drain_retention=${GUNBC_FLOOR_DRAIN_RETENTION:-unset}"
  free -b | head -2
} >"$META"

echo "m2_floor_retention_measure: $PROBE plan=$PLAN_FN log=$LOG"

START_SEC=$SECONDS
set +e
"$BIN" \
  --source-root dag \
  --source-root src/v2 \
  --plan-entry src/v2/workflow/ci_floor_plan.dag \
  --plan-function "$PLAN_FN" \
  2>&1 | tee "$LOG"
EXIT=$?
set -e
ELAPSED=$((SECONDS - START_SEC))

{
  echo "exit_code=$EXIT"
  echo "elapsed_sec=$ELAPSED"
  echo "ended_utc=$(date -u +%Y%m%dT%H%M%SZ)"
} >>"$META"

# Extract receipt lines for quick review (non-fatal).
grep -E '\[floor-drain\]|\[floor-memory\]|governor receipt|schedule-retention|FLOOR-BATCH|◷|cgroup peak' "$LOG" \
  >"$OUT_DIR/${PROBE}.extract.txt" 2>/dev/null || true

echo "m2_floor_retention_measure: done exit=$EXIT elapsed=${ELAPSED}s extract=$OUT_DIR/${PROBE}.extract.txt"
exit "$EXIT"
