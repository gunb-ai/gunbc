#!/usr/bin/env bash
# raven_witness.sh <tag> -- the carrier-grounding WITNESS run (Manager 3 msg_1870264f: fixture RED,
# positive and mutation INSIDE #14908; DESIGN 5: done means executed).
#
# PHASE 1 (positive): at this head, all five claims in
#   //v2/test/claim/compiler/infer_refinement_carrier_grounding_witness:all
# must HOLD -- above all rcg_a_non_literal_crossing_into_a_refined_field_is_accepted_and_counted,
# which refuses at base (measured: record_field_value_does_not_inhabit at exactly this shape).
# PHASE 2 (mutation control, design msg_066087bd): rename the advisory reason at its mint
# (^refinement_carrier_grounding_crossing -> ^refinement_carrier_grounding_crossing_MUTATED), regen,
# rerun: the COUNTED claim must FAIL and every other claim must still hold -- the counting is what
# the drop's population reads, so the claim that reads the count must be sensitive to the mint.
set -u
TAG="${1:-noname}"
cd "$(dirname "$0")"

MT_KB=$(awk '/MemTotal/{print $2}' /proc/meminfo)
echo "tag=$TAG MemTotal_MB=$((MT_KB/1024))"
echo "RUNNER_HEAD=$(git rev-parse HEAD)"
echo "RUNNER_BRANCH=$(git rev-parse --abbrev-ref HEAD)"
git status --short | head -8
test "$MT_KB" -ge 41943040 || { echo ABORT_MEMORY; exit 42; }

CGSELF=$(awk -F: '{ if ($1=="0") {print $3; exit} else if ($2=="memory") {print $3; exit} }' /proc/self/cgroup)
if grep -q '^0::' /proc/self/cgroup; then CGBASE=/sys/fs/cgroup; else CGBASE=$(awk -F: '$2=="memory"{print "/sys/fs/cgroup/memory"; exit}' /proc/self/cgroup); fi
[ -n "${CGBASE:-}" ] || CGBASE=/sys/fs/cgroup
CGPATH="$CGBASE${CGSELF}"
BOUND=$(cat "$CGPATH/memory.max" 2>/dev/null || cat "$CGPATH/memory.limit_in_bytes" 2>/dev/null || echo unreadable)
echo "CGROUP_INHERITED path=$CGPATH bound=$BOUND (accept 42..128 GiB; systemd-run MemoryMax=64G answers 68719476736)"
case "$BOUND" in ''|max|unreadable) echo "ABORT_CGROUP_UNBOUND bound=$BOUND"; exit 42 ;; esac
awk -v b="$BOUND" 'BEGIN { exit !(b+0 >= 45035996262 && b+0 <= 137438953472) }' || { echo "ABORT_CGROUP_RANGE bound=$BOUND"; exit 42; }
export RAVEN_CGROUP_PATH="$CGPATH"

export GITHUB_ACTIONS=true GITHUB_SHA=$(git rev-parse HEAD)
echo "env GITHUB_ACTIONS=$GITHUB_ACTIONS GITHUB_SHA=$GITHUB_SHA"
unset GUNBC_MEMORY_BUDGET_BYTES GUNBC_MEMORY_ENVELOPE_REQUIRED GUNBC_BIND_MEMORY_CGROUP_BYTES
export TMPDIR=/tmp
PATH=$PATH:/usr/local/cargo/bin

echo "=== the .dag authority carries the witness file ==="
test -f src/v2/test/claim/compiler/infer_refinement_carrier_grounding_witness.dag && echo witness_present || echo witness_MISSING
grep -c "refinement_carrier_grounding_crossing" src/v2/compiler/infer.dag | sed 's/^/authority_advisory_mint_lines=/'

echo "=== phase A: build the round host from the committed mirrors ==="
cargo build --release -p v1-compiler --bins > /tmp/b1.log 2>&1; echo "build1_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b1.log; then echo ABORT_BUILD1; grep -aE -A 14 "^error(\[|:)" /tmp/b1.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/claim_executor || { echo ABORT_NO_EXEC; exit 46; }

echo "=== the round: regenerate the mirrors from the .dag authority ==="
rm -rf target/stage0-regen-candidate
timeout 2400 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round.out 2>/tmp/round.err
echo "round_rc=$?"
tail -4 /tmp/round.out | fold -w 195
echo "--- round.err tail (20) ---"; tail -20 /tmp/round.err | fold -w 195

echo "=== phase B: rebuild from what the round installed ==="
cargo build --release -p v1-compiler --bins > /tmp/b2.log 2>&1; echo "build2_rc=$?"
if grep -qaE "^error(\[|:)" /tmp/b2.log; then echo ABORT_BUILD2; grep -aE -A 14 "^error(\[|:)" /tmp/b2.log | head -90 | fold -w 195; exit 46; fi
test -x ./target/release/gunbc || { echo ABORT_NO_GUNBC; exit 46; }

echo "=== PHASE 1: the five claims at HEAD (all must hold) ==="
timeout 2400 ./target/release/gunbc test //v2/test/claim/compiler/infer_refinement_carrier_grounding_witness:all > /tmp/wit_head.out 2>&1
echo "wit_head_rc=$?"
echo "--- wit_head verdict lines (HELD / DID NOT HOLD / NOT HELD / refused / admission) ---"
grep -aE "HELD|DID NOT HOLD|NOT HELD|admission|refused|VERDICT|verdict|claim" /tmp/wit_head.out | head -60 | fold -w 195
echo "--- wit_head tail 25 ---"; tail -25 /tmp/wit_head.out | fold -w 195

echo "=== CONTROLS: known-good claim modules on the SAME route (oar witness + fmsb) ==="
for ctl in //v2/test/claim/compiler/infer_optional_at_required_witness:all //v2/test/claim/compiler/free_monoid_structure_bound:all; do
  timeout 1200 ./target/release/gunbc test "$ctl" > /tmp/wit_ctl_$(basename ${ctl%%:*}).out 2>&1
  echo "ctl_rc[$ctl]=$?"
  grep -aE "HELD|DID NOT HOLD|NOT HELD|refused at prepare" /tmp/wit_ctl_$(basename ${ctl%%:*}).out | head -14 | fold -w 195
done

echo "=== PHASE 2: mutation control -- rename the advisory mint, regen, rerun ==="
sed -i 's/\^refinement_carrier_grounding_crossing\b/^refinement_carrier_grounding_crossing_MUTATED/g' src/v2/compiler/infer.dag
grep -c "_MUTATED" src/v2/compiler/infer.dag | sed 's/^/mutated_mint_lines=/'
rm -rf target/stage0-regen-candidate
timeout 2400 ./target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/round2.out 2>/tmp/round2.err
echo "round2_rc=$?"
cargo build --release -p v1-compiler --bins > /tmp/b3.log 2>&1; echo "build3_rc=$?"
timeout 2400 ./target/release/gunbc test //v2/test/claim/compiler/infer_refinement_carrier_grounding_witness:all > /tmp/wit_mut.out 2>&1
echo "wit_mut_rc=$?"
echo "--- wit_mut verdict lines ---"
grep -aE "HELD|DID NOT HOLD|NOT HELD|admission|refused|VERDICT|verdict|claim" /tmp/wit_mut.out | head -60 | fold -w 195
echo "--- wit_mut tail 25 ---"; tail -25 /tmp/wit_mut.out | fold -w 195

echo "=== THE READINGS ==="
for f in head mut; do
  for c in rcg_a_literal_at_a_refined_declared_return_refuses \
           rcg_a_non_literal_crossing_into_a_refined_field_is_accepted_and_counted \
           rcg_a_wrong_carrier_at_a_refined_field_still_refuses \
           rcg_an_unrefined_control_crossing_carries_no_advisory \
           rcg_b_literal_at_a_let_annotation_refuses_not_counted; do
    line=$(grep -a "$c" /tmp/wit_$f.out | tail -1 | cut -c1-160)
    echo "CLAIM[$f] $c :: ${line:-NO-OUTPUT-FOR-CLAIM}"
  done
done
echo "=== DONE tag=$TAG ==="
