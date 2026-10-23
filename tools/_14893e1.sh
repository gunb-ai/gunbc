#!/usr/bin/env bash
# #14893 @ab3279b680 E1: unit tests + NoSpans mutation + 3-driver E2E gating contract on both claims.
# H=head (clean partition -> exit 0, CONTROL); P=head+plant893-R (planted over-attribution, gate ON -> rc 1);
# G=P+fold-R+gate-R (base: gate folded away -> refusal PRINTED but exit 0). Drivers differ; all arms run the HEAD tree.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=ab3279b68044af1af59311cb6e1f0307f88a0d92
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893e 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893e
echo 51539607552 > /sys/fs/cgroup/ver893e/memory.max
RB=$(cat /sys/fs/cgroup/ver893e/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893e memory.max=$RB (read-back, bounded)"
exec 11>/sys/fs/cgroup/ver893e/cgroup.procs; echo $BASHPID >&11
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893e || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893e)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$HEAD_SHA"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893e
T=/tmp/w893e_h; P=/tmp/w893e_p; G=/tmp/w893e_g; M=/tmp/w893e_m
for d in $T $P $G $M; do git worktree prune >/dev/null 2>&1; rm -rf $d; done
for d in $T $P $G $M; do git worktree add -q --force --detach $d $HEAD_SHA || { echo WT_FAIL $d; exit 9; }; done
# P: reverse the #14872 out_of_span clamp -> planted over-attribution
(cd $P && git apply -R $REPO/tools/plant893.patch) || { echo PLANT_APPLY_FAIL; exit 9; }
grep -q 'pub out_of_span' $P/src/v1/stage0/src/cli_run.rs && { echo PLANT_NOOP; exit 9; }
grep -q 'out_of_span' $T/src/v1/stage0/src/cli_run.rs || { echo HEAD_PRECHECK_FAIL; exit 9; }
# G: P + reverse fold (ab3279b680 bin) + reverse original gate
(cd $G && git apply -R $REPO/tools/gate893b.patch && git apply -R $REPO/tools/gate893.patch) || { echo GATE_APPLY_FAIL; exit 9; }
[ "$(grep -c 'CostAccountingRefusal\|partition_exit_flags' $G/src/v1/stage0/src/bin/claim_batch.rs)" = "0" ] || { echo GATE_INCOMPLETE; exit 9; }
grep -q 'CostAccountingRefusal' $P/src/v1/stage0/src/bin/claim_batch.rs || { echo P_GATE_MISSING_UNEXPECTED; exit 9; }
echo TREES_OK
# builds
build() { # tag tree bin
  local tag=$1 tree=$2 bin=$3
  (cd $tree && cargo build --release -p v1-compiler --bin $bin 2>&1 | tail -1)
  local b=$CARGO_TARGET_DIR/release/$bin
  [ -x $b ] || { echo NO_BINARY $tag; exit 9; }
  cp $b /tmp/cb893e_$tag; sha /tmp/cb893e_$tag | sed "s/^/CB_$tag=/"; rm -f $b
}
build H $T claim_batch
# units on head (cargo test needs its own build of test harness; binary removed so it rebuilds once)
echo "=== UNIT head $(date -u +%H:%M:%S)Z"
(cd $T && cargo test --release -p v1-compiler --bin claim_batch 2>&1) | grep -aE 'test result|FAILED|panicked|running [0-9]+ test|a_refused|exit_composition' | sed 's/^/  U| /'
# mutation worktree M: flip NoSpans arm (true,false)->(false,true)
perl -0777 -i -pe 'BEGIN{$n=0} $n += s/cause: CostAccountingRefusal::NoSpans,\n        \} => \(true, false\),/cause: CostAccountingRefusal::NoSpans,\n        } => (false, true),/g; END{exit 1 unless $n==1}' $M/src/v1/stage0/src/bin/claim_batch.rs || { echo MUT_GUARD_FAIL; exit 9; }
echo MUT_APPLIED
echo "=== UNIT mutation $(date -u +%H:%M:%S)Z"
(cd $M && cargo test --release -p v1-compiler --bin claim_batch 2>&1) | grep -aE 'test result|FAILED|^failures:|panicked at|a_refused|exit_composition' | sed 's/^/  MU| /'
build P $P claim_batch
build G $G claim_batch
SHAS=$(sha256sum /tmp/cb893e_H /tmp/cb893e_P /tmp/cb893e_G | awk '{print $1}' | sort -u | wc -l)
echo "DISTINCT_CB_SHAS=$SHAS (must be 3)"
claim() { # tag fn entry
  local tag=$1 fn=$2 entry=$3
  cd $T || exit 9
  timeout 900 /tmp/cb893e_$tag --claim-run --source-root dag --source-root src/v2 --entry $entry --functions $fn > /tmp/w893e_${tag}_$fn.out 2>&1
  local rc=$?
  local cp=$(grep -aE '\[cost-partition\]' /tmp/w893e_${tag}_$fn.out | tail -1)
  echo "  [$tag $fn] rc=$rc PASS=$(grep -ac "^PASS $fn$" /tmp/w893e_${tag}_$fn.out) FAIL=$(grep -ac "^FAIL $fn$" /tmp/w893e_${tag}_$fn.out) verdict=$(printf '%s' "$cp" | grep -oE '\"state\":\"[A-Za-z]+\"' | head -1) kind=$(printf '%s' "$cp" | grep -oE '\"kind\":\"[A-Za-z]+\"' | head -1) oos=$(printf '%s' "$cp" | grep -oc 'out_of_span') BYTES=$(wc -c < /tmp/w893e_${tag}_$fn.out) $(date -u +%H:%M:%S)Z"
  cd $REPO || exit 9
}
for pair in "string_family_cast_holds dag/test/claim/string_family_cast.dag" "unicode_ident_xid_grounding_holds dag/test/claim/unicode_ident_xid_grounding.dag"; do
  set -- $pair; fn=$1; entry=$2
  [ -f $T/$entry ] || { echo NO_ENTRY $entry; exit 9; }
  for tag in H P G; do claim $tag $fn $entry; done
done
echo DONE_14893E1
