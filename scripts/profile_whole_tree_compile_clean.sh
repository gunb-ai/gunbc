#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: substrate-emitted compile-clean phase timings (realization_measurement_loop
# carrier / floor receipt resolve-split on the via-index path) retire this bash runner; until then
# it is the reproducible receipt entrypoint documented in
# docs/plans/compile-clean-whole-tree-time-diagnosis.md.
# dissolve-on: gunbc bash-emit capability (#5828 / ROADMAP shell-emission) realizes profile
# orchestration through host_effect_apply transport handlers instead of hand-rolled bash.
#
# Profile whole-tree compile-clean (locate, don't fix).
# Profile receipt: docs/plans/compile-clean-whole-tree-time-diagnosis.md
#
# Usage:
#   ./scripts/profile_whole_tree_compile_clean.sh              # build + histogram + floor receipt
#   ./scripts/profile_whole_tree_compile_clean.sh histogram    # histogram path only
#   ./scripts/profile_whole_tree_compile_clean.sh floor main   # floor receipt path only
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN_DIR="${BIN_DIR:-$ROOT/target/release}"
HIST="$BIN_DIR/compile_clean_diagnostic_histogram"
EXEC="$BIN_DIR/claim_executor"

build() {
  CTRL_BUILD_WRAP_CARGO=0 /opt/cargo/bin/cargo build -p v1-compiler --release \
    --features text_lookup_work_counter \
    --bin compile_clean_diagnostic_histogram --bin claim_executor
}

run_histogram() {
  local label="$1"
  echo "=== histogram: $label ==="
  local start end
  start=$(date +%s)
  GUNBC_FLOOR_GANTT=1 "$HIST" 2>&1 | tee "/tmp/compile_clean_hist_${label}.log"
  end=$(date +%s)
  python3 - "$((end - start))" "/tmp/compile_clean_hist_${label}.log" <<'PY'
import re, sys
wall = sys.argv[1]
log = open(sys.argv[2]).read()
def gantt(tag):
    m = re.search(rf"\[gantt\] {re.escape(tag)} t_ms=(\d+) rss_mib=(\d+)", log)
    return (int(m.group(1)), int(m.group(2))) if m else (None, None)
fe = gantt("compile.frontend.done")
rc = gantt("compile.reconcile.done")
hist = re.search(r"HISTOGRAM_ELAPSED_SECS ([0-9.]+)", log)
rss = re.search(r"HISTOGRAM_RSS_MIB (\d+)", log)
print(f"wall_s={wall} histogram_s={hist.group(1) if hist else 'NA'} peak_mib={rss.group(1) if rss else 'NA'}")
if fe[0] is not None and rc[0] is not None:
    print(f"  parse+frontend_ms={fe[0]} reconcile_ms={rc[0]-fe[0]} reconcile_rss_mib={rc[1]}")
PY
}

run_floor_receipt() {
  local label="$1"
  echo "=== floor receipt: $label ==="
  local start end
  start=$(date +%s)
  GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL=1 GUNBC_FLOOR_GANTT=1 \
    "$EXEC" \
      --source-root dag --source-root src/v2 \
      --plan-entry src/v2/workflow/ci_floor_plan.dag \
      --plan-function gunbc_ci_plan_artifact_batches \
    2>&1 | tee "/tmp/compile_clean_floor_${label}.log"
  end=$(date +%s)
  python3 - "$((end - start))" "/tmp/compile_clean_floor_${label}.log" <<'PY'
import re, sys
wall = sys.argv[1]
log = open(sys.argv[2]).read()
emit = re.search(r"compile\.emit\.done t_ms=(\d+) rss_mib=(\d+)", log)
peak = re.search(r"\[measurement\] floor peak RSS: (\d+)", log)
receipt = "receipt ok=true" in log
print(f"wall_s={wall} receipt_ok={receipt} emit_ms={emit.group(1) if emit else 'NA'} emit_rss_mib={emit.group(2) if emit else 'NA'} floor_peak_mib={int(peak.group(1))//(1024*1024) if peak else 'NA'}")
PY
}

usage() {
  echo "usage: $0 [build|histogram|floor|all] [label]"
  exit 2
}

cmd="${1:-all}"
label="${2:-main}"
case "$cmd" in
  build) build ;;
  histogram) run_histogram "$label" ;;
  floor) run_floor_receipt "$label" ;;
  all) build; run_histogram "$label"; run_floor_receipt "$label" ;;
  *) usage ;;
esac
