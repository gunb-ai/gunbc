#!/usr/bin/env bash
# STR-RC-0 mechanism check: post 100KB survival with GUNBC_EVAL_MEMO=0 only.
# Compare elapsed_ms to memo-default row (139,517 ms from clean-process receipt).
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
RESULTS_DIR="$ROOT/docs/probes/results"
mkdir -p "$RESULTS_DIR"

TIMEOUT_SEC=900
TARGET=100000
TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$RESULTS_DIR/mechanism-post-rc-str-memo0-${TARGET}-${TS}.txt"

echo "=== Dispatch metadata ==="
echo "# dispatch_head=$(git rev-parse HEAD)"
echo "# date=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo build --release -p v1-compiler --bin json_parse_scaling_probe
BIN="$ROOT/target/release/json_parse_scaling_probe"

echo "=== post-rc-str-memo0 survival target=${TARGET} (fresh process) ===" | tee "$OUT"
{
  echo "# experiment=mechanism"
  echo "# label=post-rc-str-memo0"
  echo "# target_bytes=$TARGET"
  echo "# GUNBC_EVAL_MEMO=0"
  echo "# compare_to_memo_default_ms=139517"
  echo "#"
  set +e
  timeout "$TIMEOUT_SEC" env \
    GUNBC_EVAL_MEMO=0 \
    JSON_PARSE_PROBE_MODE=survival \
    JSON_PARSE_TARGET_BYTES="$TARGET" \
    "$BIN"
  ec=$?
  set -e
  echo "# exit_code=$ec"
} | tee -a "$OUT"

echo "=== Done: $OUT ==="
