#!/usr/bin/env bash
# REPAIR MEASUREMENT, part 1 of 2 (stern-bat-401 msg_2d4b0512: "(i) Repair at the producer.
# Land the repair as its own commit with the RED: the 77 at the pre-repair commit, 0 Map-family
# after, and the per-module two-closure comparison against main").
#
# Part 1 (this script): fresh clone of the WIP ref (which carries the repair commit), regenerate
# the stage0 mirrors FROM THAT TREE to the fixed point, run the fold suite against the regenerated
# seed, and ship the changed mirrors back so they can be committed. Part 2 (the proven
# gate_two_closures_14152.sh) then measures BOTH closures against main from the COMMITTED tree --
# it needs the mirrors in the ref, so the order is forced.
#
# Why clone the WIP ref directly instead of main+file-installs (the A/B script's shape): the A/B
# script installs a HAND-PICKED file list, and doing that over a main clone silently reverted four
# upstream commits once in this lane when the base moved. The head is checked out WHOLE here.
#
# Every invocation below (claim_executor flags, suite target, filter) is copied VERBATIM from
# ab_extension_14152.sh, the runs that produced 30 passed / 0 failed at this tree.
set -u
LOGS=${LOGS:?LOGS required}
mkdir -p $LOGS
REPO=https://github.com/gunb-ai/gunbc
HEAD_REF=${HEAD_REF:-refs/heads/session/gentle-carp-16}
void_out () { echo "VOID: $1"; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ) | sed 's/^/=== BEGIN-TAR ===\n/;s/$/\n=== END-TAR ===/'; exit 0; }

HEAD_SHA=$(git ls-remote $REPO $HEAD_REF | awk '{print $1}')
[ ${#HEAD_SHA} -ge 40 ] || void_out "head sha unresolved from $HEAD_REF"
echo "PINNED head=$HEAD_SHA (from $HEAD_REF, never typed)"

D=/tmp/repair_14152
rm -rf $D
git clone -q $REPO $D > $LOGS/clone.log 2>&1 || void_out "clone failed"
( cd $D && git fetch -q --depth 1 origin $HEAD_SHA && git checkout -q FETCH_HEAD ) >> $LOGS/clone.log 2>&1 || void_out "checkout failed"
GOT=$(git -C $D rev-parse HEAD)
[ "$GOT" = "$HEAD_SHA" ] || void_out "checked out $GOT != $HEAD_SHA"
echo "tree=$GOT tracked_clean=$(git -C $D status --porcelain | wc -l) (want 0)"
# the repair must be IN the tree that regenerates, and it must be the only thing that changed
# in this function since 8d8d8aac0e1
# definition AND use, counted on the BARE name: 'let unsettled_slot_variables' matches only the
# definition (count 1); expecting 2 there VOIDs a perfectly good tree, which it did once.
SEED_HITS=$(grep -ac 'unsettled_slot_variables' $D/src/v1/04_infer.dag)
[ "$SEED_HITS" = "2" ] || void_out "repair seed hits=$SEED_HITS (want 2: definition + use) -- wrong tree"
echo "repair present in .dag: seed=$SEED_HITS/2"

cd $D
SEED_BEFORE=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
echo "BEFORE: seed=$SEED_BEFORE  (must be 8d8d8aac0e1's b471b493356ae5e8; a pre-repair mirror regenerating to itself would fake a pass)"
[ "$SEED_BEFORE" = "b471b493356ae5e8" ] || echo "NOTE: seed is $SEED_BEFORE, not b471b493356ae5e8 -- fine if main moved under the merge, the loop below still finds the fixed point"

cargo build --release -p v1-compiler --bin claim_executor > $LOGS/ce_build.log 2>&1
echo "claim_executor build rc=$?"
grep -E "^error|error\[|Finished" $LOGS/ce_build.log | tail -3
[ -f target/release/claim_executor ] || void_out "no claim_executor binary (its rc=0 would be a false green)"

ROUND=1; INST=1
while [ $ROUND -le 4 ] && [ "$INST" != "0" ]; do
  timeout 2400 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_$ROUND.log 2>&1
  echo "=== regen round $ROUND rc=$?"
  grep -a "^required-regen" $LOGS/rr_$ROUND.log | head -2 | cut -c1-300
  if [ ! -d target/stage0-regen-candidate/src ]; then
    echo "NO CANDIDATE: the receipt is the answer. Hard diagnostics verbatim:"
    grep -aE "^type mismatch|^call shape|arm decides|not found in|no field|effect summary" $LOGS/rr_$ROUND.log | head -14 | cut -c1-300
    void_out "regen produced no candidate at round $ROUND"
  fi
  INST=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f src/v1/stage0/src/$f ]; then
      if ! cmp -s target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f; then
        cp target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f
        INST=$((INST+1)); echo "   installed $f"
      fi
    else
      echo "   CANDIDATE NOT IN SEED (reported, never committed): $f"
    fi
  done
  echo "   round $ROUND installed=$INST"
  ROUND=$((ROUND+1))
done
[ "$INST" = "0" ] || void_out "regen did not converge in 4 rounds"
SEED_AFTER=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)
echo "CONVERGED. seed: $SEED_BEFORE -> $SEED_AFTER"
# THE REPAIR MUST HAVE MOVED THE SEED. If the emitted mirror is byte-identical to the pre-repair
# one, the .dag change did not reach the emitter and every number after this measures the old build.
[ "$SEED_AFTER" != "$SEED_BEFORE" ] || echo "WARNING: seed unchanged by the repair -- check the emitter reads 04_infer.dag"
echo "seed: fold=$(grep -ac UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) refusal=$(grep -ac '(first argument)' src/v1/stage0/src/v1_compiler_infer.rs) seedVar=$(grep -ac 'unsettled_slot_variables' src/v1/stage0/src/v1_compiler_infer.rs)"

# THE SUITE, against the regenerated seed. Verbatim from the A/B script: the tests live in
# v1-compiler-tests as a LIB MODULE, not as a --test target; the wrong invocation reported an empty
# suite as a pass once.
cargo build --release -p v1-compiler-tests --tests > $LOGS/tb.log 2>&1
echo "test build rc=$?"; grep -E "^error|error\[" $LOGS/tb.log | head -6 | cut -c1-200
timeout 1200 cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 \
  | grep -E "^test |test result|^error\[|panicked|assertion" \
  | sed -E 's/^test cross_coproduct_generic_locate_test:://' > $LOGS/suite.txt
[ -s $LOGS/suite.txt ] || void_out "suite produced no output: FAILURE, not a pass"
cat $LOGS/suite.txt
OKN=$(grep -c 'ok$' $LOGS/suite.txt); BADN=$(grep -c FAILED $LOGS/suite.txt)
echo "suite passed=$OKN failed=$BADN"
[ $((OKN + BADN)) -gt 0 ] || void_out "suite ran ZERO tests (filtered everything) -- not a pass"

# SHIP: the changed mirrors, named by git, never by me. gzip+base64; emitted in its OWN block
# BEFORE anything big, because a 1.6 MB single line once hit ctrl-build's cap and lost a dispatch.
CHANGED=$(git status --porcelain -- src/v1/stage0/src | awk '{print $2}')
echo "git-named changed mirrors: $(echo $CHANGED | tr '\n' ' ')"
if echo "$CHANGED" | grep -qE 'cli_run|module_path_index'; then
  void_out "changed list contains a HAND_MAINTAINED mirror -- refusing to ship"
fi
( cd src/v1/stage0/src && tar -c $(for f in $CHANGED; do basename $f; done) 2>/dev/null ) | gzip -9 | base64 -w0 > $LOGS/stage0_regen.b64
echo "staged tar: $(wc -c < $LOGS/stage0_regen.b64) base64 bytes"
echo "=== BEGIN-TAR ==="; cat $LOGS/stage0_regen.b64; echo; echo "=== END-TAR ==="
# compact logs only: the full self-host log is ~1.2 MB and has eaten a dispatch before
( cd $LOGS && tar -c suite.txt rr_1.log rr_2.log 2>/dev/null | gzip | base64 -w0 ) | sed 's/^/=== BEGIN-LOGS ===\n/;s/$/\n=== END-LOGS ===/'
echo "PART1 DONE seed=$SEED_AFTER suite=$OKN/$BADN changed=$(echo $CHANGED | tr '\n' ' ')"
exit 0
