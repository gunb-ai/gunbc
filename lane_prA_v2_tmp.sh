#!/bin/bash
# Scratch (never lands; mirrors untracked): prA lane v2. v1 lesson: claim_batch built
# PRE-regen ran the old mirror -> values arms rc=1 by construction; cap hit before patch emit.
# v2 order: build -> regen to fixed point -> WITNESSES -> PATCH FIRST -> rebuild bins -> battery.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current
git clean -x -d --force >/dev/null

echo "=== 1. BUILD ==="
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc --bin claim_batch 2>&1 | tail -1

echo "=== 2. REGEN to fixed point ==="
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen$ROUND.log 2>&1
  echo "regen-round-cost rc=$?"; grep -aE "changed_paths|RebuildScopeRefused" /tmp/regen$ROUND.log | head -2
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet > /tmp/heal$ROUND.log 2>&1
  echo "heal rc=$?"
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet > /tmp/regenw$ROUND.log 2>&1
  echo "seed-regen rc=$?"
  WRITTEN=$(git status --porcelain | wc -l | tr -d ' ')
  echo "round $ROUND dirty=$WRITTEN"
  if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
  PREV=$WRITTEN
done

echo "=== 3. WITNESSES ==="
echo "find_prefix_binding_power: $(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs)"
echo "literal min_bp arms: $(grep -c 'min_bp: 1[0-9]' src/v1/stage0/src/v1_compiler_parse.rs)"
echo "prefix_operators std_syntax: $(grep -c prefix_operators src/v1/stage0/src/std_syntax.rs)"
git diff --stat -- src/v1/stage0/src | tail -5

echo "=== 4. PATCH FIRST (before anything can hit the cap) ==="
git add -A src/v1/stage0/src
echo "BEGIN-PATCH"
git diff --cached | base64 -w0
echo
echo "END-PATCH"

echo "=== 5. REBUILD bins over regenerated mirror ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1

echo "=== 6. BATTERY (refusal text visible) ==="
echo "--- 6a values arms ---"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_values.dag --function unary_prefix_value_arms_hold 2>&1 | grep -aE "want|✗|refus|error" | head -4
echo "values-arms rc=${PIPESTATUS[0]}"
echo "--- 6b values emitted-text ---"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_values.dag --function unary_prefix_emitted_text_follows_the_tree 2>&1 | grep -aE "want|✗|refus|error" | head -4
echo "values-emitted rc=${PIPESTATUS[0]}"
echo "--- 6c agreement (14 arms) ---"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports 2>&1 | grep -aE "DISAGREE|✗|refus|error" | head -6
echo "agreement rc=${PIPESTATUS[0]}"
echo "--- 6d roster acceptance (re-spelled sites regression) ---"
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag --function v1_interpreter_primitive_roster_acceptance_contract_holds 2>&1 | grep -aE "✗|refus|error" | head -4
echo "roster rc=${PIPESTATUS[0]}"
echo "=== DONE ==="
