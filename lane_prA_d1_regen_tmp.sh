#!/bin/bash
# Scratch (never lands): D1c = regenerate the seed mirrors over the merged .dag sources, with the
# bootstrap binary = main's mirror (it parses main's corpus, which my ed1719-era mirror could not)
# plus the hand-kept namespace_baseline.rs. A round that REFUSES is a hard failure, never a
# "fixed point": witnesses gate the exit.
set -uo pipefail
PIN=d1661d1147fe67ea12a9af04e4ba645030194643
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then
  echo "PIN: runner was at $CUR; checking out $PIN"
  git checkout -f -q "$PIN" || { echo "PIN-FAILED"; exit 3; }
fi
git rev-parse HEAD | cut -c1-11
test "$(git rev-parse HEAD)" = "$PIN" || { echo "PIN-MISMATCH"; exit 3; }

MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current

echo "=== BUILD (bootstrap binary = main's mirror + hand baseline) ==="
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | tail -2

echo "=== REGEN to fixed point ==="
BASELINE=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
echo "baseline mirror-dirty=$BASELINE"
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen$ROUND.log 2>&1
  RRC=$?
  echo "regen-round-cost rc=$RRC"; grep -aE "changed_paths|REFUSED|refused|error" /tmp/regen$ROUND.log | head -4
  if [ "$RRC" != 0 ]; then echo "ROUND-REFUSED (regen-round-cost)"; tail -6 /tmp/regen$ROUND.log; fi
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet > /tmp/heal$ROUND.log 2>&1
  HRC=$?
  echo "heal rc=$HRC"
  if [ "$HRC" != 0 ]; then echo "ROUND-REFUSED (heal)"; tail -6 /tmp/heal$ROUND.log; fi
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet > /tmp/regenw$ROUND.log 2>&1
  SRC=$?
  echo "seed-regen rc=$SRC"
  if [ "$SRC" != 0 ]; then echo "ROUND-REFUSED (seed-regen)"; tail -8 /tmp/regenw$ROUND.log; fi
  if [ "$RRC$HRC$SRC" = "000" ]; then
    WRITTEN=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
    echo "round $ROUND mirror-dirty=$WRITTEN"
    if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
    PREV=$WRITTEN
  else
    echo "round $ROUND: refusal(s); continuing (idempotent commands)"
  fi
done

echo "=== WITNESSES (hard gate) ==="
P=src/v1/stage0/src
W1=$(grep -c find_prefix_binding_power $P/v1_compiler_parse.rs)
W2=$(grep -c declared_postfix_gate_admits $P/v1_compiler_parse.rs)
W3=$(grep -cE '21 < min_bp|21 >= min_bp|13 < min_bp|14 <= min_bp|min_bp: 12' $P/v1_compiler_parse.rs)
W4=$(grep -c postfix_operators $P/std_syntax.rs)
W5=$(grep -c PostfixOperatorSpec $P/extdeps_languages_dag_syntax.rs)
W6=$(grep -c PostfixOperatorSpec $P/extdeps_languages_rust_syntax.rs)
W7=$(grep -c decode_environment_wire $P/namespace_baseline.rs)
W8=$(grep -c migrate_syntax_spec_wire $P/namespace_baseline.rs)
echo "W1 find_prefix=$W1 W2 declared_postfix=$W2 W3 literal-gates=$W3 W4 std_syntax-postfix=$W4 W5 dag-rows=$W5 W6 rust-rows=$W6 W7 decode_wire=$W7 W8 migration=$W8"
if [ "$W1" -ge 2 ] && [ "$W2" -ge 5 ] && [ "$W3" = 0 ] && [ "$W4" -ge 2 ] && [ "$W5" -ge 3 ] && [ "$W6" -ge 3 ] && [ "$W7" -ge 2 ] && [ "$W8" -ge 2 ]; then
  echo "WITNESSES-OK"
else
  echo "WITNESSES-FAILED"
fi
git diff --stat -- src/v1/stage0/src | tail -8

echo "=== PATCH (all mirror changes incl. untracked additions) ==="
git add -u src/v1/stage0/src
echo "BEGIN-PATCH"; git diff --cached | base64 -w0; echo; echo "END-PATCH"
git reset -q -- src/v1/stage0/src
echo "=== DONE ==="
