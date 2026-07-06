#!/usr/bin/env bash
# Wall-clock parse probes for operator mechanism discrimination (sunny-wren-799).
set -euo pipefail
ROOT="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
cd "$ROOT"
GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
ENTRY=src/v2/test/claim/manual/parse_mechanism_probe.dag
SR=(--source-root src/v2 --source-root dag)
CAP="${CAP:-600}"

run_probe() {
  local label="$1" path="$2"
  local start end elapsed rc
  start=$(date +%s)
  set +e
  timeout "${CAP}s" "$GUNBC" run "${SR[@]}" --entry "$ENTRY" \
    --function pmp_tokenize_parse_path \
    --arg path "$path" >/dev/null 2>&1
  rc=$?
  set -e
  end=$(date +%s)
  elapsed=$((end - start))
  local verdict="COMPLETED"
  if [[ $rc -eq 124 ]]; then verdict="NO_VERDICT@${CAP}s"
  elif [[ $rc -ne 0 ]]; then verdict="FAIL(rc=$rc)"; fi
  printf '%s\t%s\t%s\t%s\n' "$label" "$path" "$verdict" "${elapsed}s"
}

run_memo() {
  local label="$1" path="$2"
  "$GUNBC" run "${SR[@]}" --entry "$ENTRY" \
    --function pmp_memo_stats_report_path \
    --arg path "$path" 2>/dev/null | tail -1
}

echo "=== (1) FLAT-SIZE CONTROL (tokenize+parse) ==="
for f in flat_1p5kb flat_3kb flat_6kb flat_12kb; do
  run_probe "flat-size" "src/v2/test/fixture/parse_mechanism/${f}.dag"
done

echo "=== (2) NESTING-DEPTH CURVE (tokenize+parse) ==="
for d in 1 2 3 4; do
  run_probe "nest-depth" "src/v2/test/fixture/parse_mechanism/nest_d${d}.dag"
done

echo "=== (3) ELSE-IF CHAIN CURVE (tokenize+parse) ==="
for k in 4 8 16 32; do
  run_probe "elseif-k" "src/v2/test/fixture/parse_mechanism/elseif_k${k}.dag"
done

echo "=== (4) MEMO-COUNTER through_decimal_compare ==="
BYTES=$(wc -c < src/v2/test/fixture/integer_census_trunc/through_decimal_compare.txt)
echo -n "through_decimal_compare\t${BYTES}B\tmemo_report="
run_memo "decimal" src/v2/test/fixture/integer_census_trunc/through_decimal_compare.txt
start=$(date +%s)
set +e
timeout "${CAP}s" "$GUNBC" run "${SR[@]}" --entry "$ENTRY" \
  --function pmp_tokenize_parse_path \
  --arg path src/v2/test/fixture/integer_census_trunc/through_decimal_compare.txt >/dev/null 2>&1
rc=$?
set -e
end=$(date +%s)
verdict="COMPLETED"; [[ $rc -eq 124 ]] && verdict="NO_VERDICT@${CAP}s"
[[ $rc -ne 0 && $rc -ne 124 ]] && verdict="FAIL(rc=$rc)"
echo "through_decimal_compare parse wall-clock: ${verdict} $((end-start))s"
