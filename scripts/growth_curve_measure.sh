#!/usr/bin/env bash
# SCAFFOLD (§7 seed-retained HAND-SHELL — authority: sleek-wolf-190 global_bare cost-shape receipt;
# receipt: PR #6743 / quiet-gull-833 defork blowup investigation).
# 🟡 dissolve-on: cost-shape receipt landed (located root + n-vs-time growth curve on bounded
# corpus); namespace unblock lands cost fix then de-fork from defork-preserve-quiet-gull-833.
# DELETE WHEN dissolved: this script and `GROWTH_CURVE_MEASURE_SCAFFOLD_MARKER`.
# Receipt: `rg GROWTH_CURVE_MEASURE_SCAFFOLD_MARKER scripts/growth_curve_measure.sh` == 1
# Local investigation helper only — NOT a CI gate (§3: no shell-as-authority).
# §5: `GUNBC_GLOBAL_BARE_RECEIPT_BASELINE_MERGE` refuses green compile after receipt.
GROWTH_CURVE_MEASURE_SCAFFOLD_MARKER=growth_curve_measure_receipt
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
  echo "wall_s=${elapsed} exit=${ec} (non-zero expected — receipt mode refuses green compile)"
  echo
}

run_point "std-algebra" "src/v2/std/algebra.dag"
run_point "v2-infer" "src/v2/compiler/04_infer.dag"
run_point "ci-spec" "dag/gunbc/ci_spec.dag"
