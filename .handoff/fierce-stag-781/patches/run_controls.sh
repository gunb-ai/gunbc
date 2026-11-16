#!/bin/bash
BIN=/tmp/fierce-stag-781-target/release/gunbc
ENTRY=$1
shift
for fn in "$@"; do
  out=$("$BIN" run --claim-run --entry "$ENTRY" --function "$fn" --source-root dag --source-root src/v2 2>&1)
  line=$(echo "$out" | grep -E 'PASS |FAIL |error:|error\b' | head -3 | tr '\n' ' | ')
  if echo "$out" | grep -q "PASS $fn"; then echo "PASS  $fn";
  elif echo "$out" | grep -q "FAIL $fn"; then echo "FAIL  $fn :: $line";
  else echo "ERR   $fn :: $line"; fi
done
