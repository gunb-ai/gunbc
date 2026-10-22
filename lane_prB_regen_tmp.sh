#!/bin/bash
# Scratch (never lands): lane B mirror round for the data-initializer line rule. Bootstrap binary
# builds from the branch's own mirrors (green tip base). A REFUSED round is a hard failure.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
P=src/v1/stage0/src

echo "=== HEAD ==="; git rev-parse --short HEAD; git branch --show-current

echo "=== PRE-FIX MIRROR STATE (must show 0 skip_newlines inside parse_data_after_kw) ==="
awk '/fn parse_data_after_kw/,/^}/' $P/v1_compiler_parse.rs | grep -c 'skip_newlines' | sed 's/^/PRE skip_newlines-in-data-fn=/'
awk '/fn parse_data_after_kw/,/^}/' $P/v1_compiler_parse.rs | grep -n 'heads_only\|parse_expr(' | head -4 | cut -c1-120

echo "=== BUILD bootstrap ==="
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | tail -2

BASELINE=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
echo "baseline mirror-dirty=$BASELINE"
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen$ROUND.log 2>&1
  RRC=$?; echo "regen-round-cost rc=$RRC"; grep -aE "changed_paths|REFUSED|refused|error" /tmp/regen$ROUND.log | head -4
  [ "$RRC" != 0 ] && { echo "ROUND-REFUSED (regen-round-cost)"; tail -6 /tmp/regen$ROUND.log; }
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet > /tmp/heal$ROUND.log 2>&1
  HRC=$?; echo "heal rc=$HRC"
  [ "$HRC" != 0 ] && { echo "ROUND-REFUSED (heal)"; tail -6 /tmp/heal$ROUND.log; }
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet > /tmp/regenw$ROUND.log 2>&1
  SRC=$?; echo "seed-regen rc=$SRC"
  [ "$SRC" != 0 ] && { echo "ROUND-REFUSED (seed-regen)"; tail -8 /tmp/regenw$ROUND.log; }
  if [ "$RRC$HRC$SRC" = "000" ]; then
    WRITTEN=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
    echo "round $ROUND mirror-dirty=$WRITTEN"
    if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
    PREV=$WRITTEN
  else
    echo "round $ROUND: refusal(s); continuing"
  fi
done

echo "=== WITNESSES (hard gate) ==="
IN_FN=$(awk '/fn parse_data_after_kw/,/^}/' $P/v1_compiler_parse.rs | grep -c 'skip_newlines')
HEADS=$(awk '/fn parse_data_after_kw/,/^}/' $P/v1_compiler_parse.rs | grep -c 'heads_only')
DIRTY=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
echo "W1 skip_newlines-in-data-fn=$IN_FN (want >=2)  W2 heads_only-present=$HEADS (want >=1)  dirty=$DIRTY"
git status --porcelain -uno -- src/v1/stage0/src | head -8
if [ "$IN_FN" -ge 2 ] && [ "$HEADS" -ge 1 ]; then echo "WITNESSES-OK"; else echo "WITNESSES-FAILED"; fi
git diff --stat -- src/v1/stage0/src | tail -8

echo "=== PATCH (all mirror changes incl. untracked additions) ==="
git add -u src/v1/stage0/src
echo "BEGIN-PATCH"; git diff --cached | base64 -w0; echo; echo "END-PATCH"
git reset -q -- src/v1/stage0/src
echo "=== DONE ==="
