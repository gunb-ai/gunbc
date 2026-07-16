#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cargo build --release -p v1-compiler --bin gunbc >/dev/null 2>&1
GUNBC="$PWD/target/release/gunbc"
ROOTS=(--source-root dag --source-root src/v2 --source-root src/v1)

run_point() {
  local label="$1"
  local entry="$2"
  local outdir="/tmp/gunbc-receipt-${label}"
  rm -rf "$outdir"
  local start end elapsed
  start=$(date +%s.%N)
  echo "=== $label entry=$entry ==="
  set +e
  env GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE=1 \
    "$GUNBC" compile --target dag --output-dir "$outdir" "${ROOTS[@]}" --entry "$entry" 2>&1 \
    | rg "global-bare-receipt|compiling [0-9]+|indexed"
  local ec=${PIPESTATUS[0]}
  set -e
  end=$(date +%s.%N)
  elapsed=$(python3 -c "print(round($end - $start, 3))")
  echo "wall_s=${elapsed} exit=${ec}"
  echo
}

run_point "std-algebra" "src/v2/std/algebra.dag"
run_point "v2-infer" "src/v2/compiler/04_infer.dag"
run_point "ci-spec" "dag/gunbc/ci_spec.dag"
