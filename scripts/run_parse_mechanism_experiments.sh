#!/usr/bin/env bash
set -euo pipefail
ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
cd "$ROOT"
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
ENTRY=src/v2/test/claim/manual/parse_mechanism_probe.dag
SR=(--source-root src/v2 --source-root dag)
CAP="${CAP:-600}"

run_fn() {
  local label="$1" fn="$2"
  local start end elapsed rc
  start=$(date +%s)
  set +e
  timeout "${CAP}s" "$GUNBC" run "${SR[@]}" --entry "$ENTRY" --function "$fn" >/dev/null 2>&1
  rc=$?
  set -e
  end=$(date +%s)
  elapsed=$((end - start))
  local verdict="COMPLETED"
  if [[ $rc -eq 124 ]]; then verdict="NO_VERDICT@${CAP}s"
  elif [[ $rc -ne 0 ]]; then verdict="FAIL(rc=$rc)"; fi
  printf '%s\t%s\t%s\t%s\n' "$label" "$fn" "$verdict" "${elapsed}s"
}

echo "=== BEFORE: (1) FLAT-SIZE ==="
run_fn flat-1.5kb exp_flat_1p5kb_parse_holds
run_fn flat-3kb exp_flat_3kb_parse_holds
run_fn flat-6kb exp_flat_6kb_parse_holds
run_fn flat-12kb exp_flat_12kb_parse_holds

echo "=== BEFORE: (2) NESTING-DEPTH ==="
run_fn nest-d1 exp_nest_d1_parse_holds
run_fn nest-d2 exp_nest_d2_parse_holds
run_fn nest-d3 exp_nest_d3_parse_holds
run_fn nest-d4 exp_nest_d4_parse_holds

echo "=== BEFORE: (3) ELSE-IF CHAIN ==="
run_fn elseif-k4 exp_elseif_k4_parse_holds
run_fn elseif-k8 exp_elseif_k8_parse_holds
run_fn elseif-k16 exp_elseif_k16_parse_holds
run_fn elseif-k32 exp_elseif_k32_parse_holds

echo "=== BEFORE: (4) MEMO through_decimal (optional) ==="
set +e
out=$("$GUNBC" run "${SR[@]}" --entry "$ENTRY" --function pmp_through_decimal_memo_report 2>&1 | tail -3)
set -e
echo "memo_report: $out"
run_fn through-decimal exp_through_decimal_parse_holds
