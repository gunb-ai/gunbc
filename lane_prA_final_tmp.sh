#!/bin/bash
# Scratch (never lands): lane A final at merged head d305b166a27. Part 1 = mirror round
# (fixed point over present files); Part 2 = the claim battery + decode tests at that head.
# A REFUSED round is a hard failure, never a "fixed point".
set -uo pipefail
PIN=dfd432b29865af9d11311c98efa49c19e1ced161
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR; checking out $PIN"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
test "$(git rev-parse HEAD)" = "$PIN" || { echo PIN-MISMATCH; exit 3; }
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
P=src/v1/stage0/src
echo "=== HEAD ==="; git rev-parse --short HEAD

echo "=== BUILD (bootstrap = merged mirrors + hand baseline) ==="; date -u +%H:%M:%S
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc --bin claim_batch 2>&1 | tail -2

echo "=== ROUND over present files ==="
BASELINE=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
echo "baseline mirror-dirty=$BASELINE"
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"; date -u +%H:%M:%S
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen$ROUND.log 2>&1
  RRC=$?; echo "regen-round-cost rc=$RRC"; grep -aE "changed_paths|REFUSED|error" /tmp/regen$ROUND.log | head -3
  [ "$RRC" != 0 ] && { echo "ROUND-REFUSED (regen-round-cost)"; tail -6 /tmp/regen$ROUND.log; }
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet > /tmp/heal$ROUND.log 2>&1
  HRC=$?; echo "heal rc=$HRC"; [ "$HRC" != 0 ] && { echo "ROUND-REFUSED (heal)"; tail -6 /tmp/heal$ROUND.log; }
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet > /tmp/regenw$ROUND.log 2>&1
  SRC=$?; echo "seed-regen rc=$SRC"; [ "$SRC" != 0 ] && { echo "ROUND-REFUSED (seed-regen)"; tail -8 /tmp/regenw$ROUND.log; }
  if [ "$RRC$HRC$SRC" = "000" ]; then
    WRITTEN=$(git status --porcelain -uno -- src/v1/stage0/src | wc -l | tr -d ' ')
    echo "round $ROUND mirror-dirty=$WRITTEN"
    if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
    PREV=$WRITTEN
  else
    echo "round $ROUND: refusal(s); continuing"
  fi
done

echo "=== WITNESSES ==="
W1=$(grep -c 'find_prefix_binding_power' $P/v1_compiler_parse.rs)
W2=$(grep -c 'declared_postfix_gate_admits' $P/v1_compiler_parse.rs)
W3=$(grep -cE 'min_bp: 12\b|21 < min_bp|13 < min_bp|14 <= min_bp' $P/v1_compiler_parse.rs)
W4=$(grep -c 'PostfixOperatorSpec' $P/extdeps_languages_rust_syntax.rs)
W5=0 # data-skip is lane B (not at this head)
echo "W1 find_prefix=$W1(want3) W2 declared_postfix=$W2(want8) W3 literal-gates=$W3(want0) W4 rust-rows=$W4(want4)"

echo "=== BATTERY at this head ==="
run_claim() {
  echo "STEP claim $1"; date -u +%H:%M:%S
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag     --entry "$2" --function "$3" > /tmp/c9_$1.log 2>&1
  echo "claim@$1 rc=$?"
  grep -am1 'FAIL\|resolve failed' /tmp/c9_$1.log | cut -c1-200
}
run_claim values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm
echo "=== decode roundtrip suite ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|test result' | head -8 | cut -c1-200

echo "=== PATCH ==="
git add -u src/v1/stage0/src
echo "BEGIN-PATCH"; git diff --cached | base64 -w0; echo; echo "END-PATCH"
git reset -q -- src/v1/stage0/src
echo "=== DONE ==="
