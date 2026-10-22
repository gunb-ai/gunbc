#!/usr/bin/env bash
# #14604 @ 0a934b479b20 — dispatch 1: head build; 6 wet claims by name (one invocation);
# coherence witness 7 claims by name; belt_capture_intent sibling control. All --wet, enrolled route.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=0a934b479b206dcc5ca4d5f169d4b99be9e136ed
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verf04 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verf04
echo 51539607552 > /sys/fs/cgroup/verf04/memory.max
RB=$(cat /sys/fs/cgroup/verf04/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verf04 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verf04/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
WET=dag/test/claim/roadmap/roadmap_belt_recapture_wet_witness.dag
COH=dag/test/claim/floor/floor_expected_red_coherence_witness.dag
SIB=dag/test/claim/roadmap/belt_capture_intent_wet_witness.dag
git fetch -q origin +refs/pull/14604/head:refs/verifier/f04 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f04)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f04) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/wf04 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wf04; git worktree add --detach /tmp/wf04 $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/wf04 ] || { echo WT_FAIL; exit 9; }
cd /tmp/wf04 || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "WT_NOT_HEAD"; exit 9; }
for f in $WET $COH $SIB; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
WETN=$(grep -oE '^test fn [a-z_0-9]+' $WET | sed 's/^test fn //' | paste -sd,)
COHN=$(grep -oE '^test fn [a-z_0-9]+' $COH | sed 's/^test fn //' | paste -sd,)
SIBN=$(grep -oE '^test fn [a-z_0-9]+' $SIB | sed 's/^test fn //' | head -1)
echo "HEADSHA=$OID"
echo "WETNAMES=$WETN"
echo "COHNAMES=$COHN"
echo "SIBNAME=$SIBN"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf04
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf04/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
runf() { # entry names tag
  local entry=$1 names=$2 tag=$3
  timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $entry --functions "$names" > /tmp/o04_${tag}.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/o04_${tag}.out) FAIL=$(grep -ac '^FAIL ' /tmp/o04_${tag}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/o04_${tag}.out)"
  for n in $(echo $names | tr ',' ' '); do
    echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/o04_${tag}.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/o04_${tag}.out) UNOBS=$(grep -ac "^UNOBSERVED $n\$" /tmp/o04_${tag}.out)"
  done
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/o04_${tag}.out | head -4 | sed 's/^/    V| /' | cut -c1-220
}
echo "=== A. 6 belt recapture claims, wet:"
runf $WET "$WETN" wet6
echo "=== B. coherence witness, wet:"
runf $COH "$COHN" coh7
echo "=== C. sibling control (belt_capture_intent), wet:"
runf $SIB "$SIBN" sib
git status --porcelain | head -3
echo "DONE_14604_D1 $(date -u +%H:%M:%S)Z"
