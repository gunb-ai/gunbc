#!/usr/bin/env bash
# Mechanism probes #2/#3/#4 — sunny-wren-799 (no --arg; per-function entry points).
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
  timeout "${CAP}s" "$GUNBC" run "${SR[@]}" --entry "$ENTRY" --function "$fn" --claim-run >/dev/null 2>&1
  rc=$?
  set -e
  end=$(date +%s)
  elapsed=$((end - start))
  local verdict="COMPLETED"
  if [[ $rc -eq 124 ]]; then verdict="NO_VERDICT@${CAP}s"
  elif [[ $rc -ne 0 ]]; then verdict="FAIL(rc=$rc)"; fi
  printf '%s\t%s\t%ss\n' "$label" "$verdict" "$elapsed"
}

run_list_fn() {
  local label="$1" fn="$2"
  local out
  out=$("$GUNBC" run "${SR[@]}" --entry "$ENTRY" --function "$fn" 2>/dev/null | tail -1)
  printf '%s\t%s\n' "$label" "$out"
}

python3 scripts/parse_mechanism_synthetics.py

echo "=== #2 NESTING DEPTH (tokenize+parse) ==="
for d in 1 2 3 4; do
  run_fn "nest_d${d}" "exp_nest_d${d}_parse_holds"
done

echo "=== #3 ELSE-IF CHAIN (tokenize+parse) ==="
for k in 4 8 16 32; do
  run_fn "elseif_k${k}" "exp_elseif_k${k}_parse_holds"
done

echo "=== #4 MEMO-COUNTER through_decimal_compare (hits misses lookup_calls) ==="
BYTES=$(wc -c < src/v2/test/fixture/integer_census_trunc/through_decimal_compare.txt)
echo -n "bytes=${BYTES}B memo="
run_list_fn "through_decimal" "pmp_through_decimal_memo_report"
run_fn "through_decimal_parse" "exp_through_decimal_parse_holds"

echo "=== REFERENCE through_compose_types ==="
BYTES=$(wc -c < src/v2/test/fixture/integer_census_trunc/through_compose_types.txt)
echo -n "bytes=${BYTES}B memo="
run_list_fn "through_compose" "pmp_through_compose_memo_report"
# shellcheck disable=SC2016
run_fn "through_compose_parse" "exp_through_compose_parse_holds"
