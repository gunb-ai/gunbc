#!/usr/bin/env bash
# #14604 @ 0a934b479b20 — dispatch 2: BASE arm (merge-base d577b1856cd1, same head-built binary):
# 3 claims expect FAIL, sibling control PASS. Then the RENAME mutation: claim 3 stores its capture
# under the CORRECT basename (base_sha -> worker_sha at the single line-232 anchor) -- claim 3 must
# FAIL (it earns the absent detail precisely because the capture is misaddressed); others hold.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=0a934b479b206dcc5ca4d5f169d4b99be9e136ed
BASE=d577b1856cd1fe9b1207a94eb108fa0eed4c92b3
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verf04b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verf04b
echo 51539607552 > /sys/fs/cgroup/verf04b/memory.max
RB=$(cat /sys/fs/cgroup/verf04b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verf04b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verf04b/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
WET=dag/test/claim/roadmap/roadmap_belt_recapture_wet_witness.dag
SIB=dag/test/claim/roadmap/belt_capture_intent_wet_witness.dag
C1=the_recapture_of_a_committed_attempt_tree_is_minted_by_real_execution
C2=a_clean_worktree_whose_head_is_not_the_recorded_submission_refuses_by_real_execution
C3=a_capture_claiming_this_head_stored_under_another_basename_refuses_by_real_execution
SIBF=a_kill_after_the_scm_save_does_not_double_capture
git fetch -q origin +refs/pull/14604/head:refs/verifier/f04b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f04b)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f04b) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/wf04b 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wf04b; git worktree add --detach /tmp/wf04b $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/wf04b ] || { echo WT_FAIL; exit 9; }
cd /tmp/wf04b || exit 9
[ "$(git cat-file blob $OID:$WET | sha256sum | cut -d' ' -f1)" = "$(sha $WET)" ] || { echo "NOT_HEAD_BLOB WET -- STOP"; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf04
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf04/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
cp -f $WET /tmp/wet_head_orig
runf() { # dir entry names tag
  local dir=$1 entry=$2 names=$3 tag=$4
  cd $dir || { echo "  [$tag] DIR_FAIL"; return; }
  timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $entry --functions "$names" > /tmp/p04_${tag}.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/p04_${tag}.out) FAIL=$(grep -ac '^FAIL ' /tmp/p04_${tag}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/p04_${tag}.out)"
  for n in $(echo $names | tr ',' ' '); do
    echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/p04_${tag}.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/p04_${tag}.out) UNOBS=$(grep -ac "^UNOBSERVED $n\$" /tmp/p04_${tag}.out)"
  done
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/p04_${tag}.out | head -4 | sed 's/^/    V| /' | cut -c1-220
}
echo "=== A. BASE $BASE (expect C1 C2 C3 FAIL; sibling PASS with head binary):"
git worktree remove --force /tmp/bf04 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/bf04; git worktree add --detach /tmp/bf04 $BASE 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/bf04 ] || { echo BASE_WT_FAIL; exit 9; }
runf /tmp/bf04 $WET "$C1,$C2,$C3" base
runf /tmp/bf04 $SIB "$SIBF" base_sib
echo "=== B. RENAME mutation on claim 3 (store under CORRECT basename base_sha->worker_sha at line 232 anchor):"
cd /tmp/wf04b || exit 9
python3 - <<'PY'
p='dag/test/claim/roadmap/roadmap_belt_recapture_wet_witness.dag'
s=open(p).read()
old='shell.Remove.FileForce(path: join([submissions, "/", submission_capture_basename(head_sha: worker_sha)], "") as FilePath).success\n    && Filesystem.Write(\n      path: join([submissions, "/", submission_capture_basename(head_sha: base_sha)], ""),'
new='shell.Remove.FileForce(path: join([submissions, "/", submission_capture_basename(head_sha: worker_sha)], "") as FilePath).success\n    && Filesystem.Write(\n      path: join([submissions, "/", submission_capture_basename(head_sha: worker_sha)], ""),'
n=s.count(old)
print("  rename anchor count:",n)
assert n==1, n
open(p,'w').write(s.replace(old,new))
PY
grep -c 'submission_capture_basename(head_sha: base_sha)' $WET | sed 's/^/  base_sha-basename-remaining: /'
runf /tmp/wf04b $WET "$C1,$C2,$C3" rename
echo "=== C. restore byte-exact + re-PASS claim 3:"
cp -f /tmp/wet_head_orig $WET
[ "$(sha $WET)" = "$(sha /tmp/wet_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"
runf /tmp/wf04b $WET "$C3" post
git status --porcelain | head -3
echo "DONE_14604_D2 $(date -u +%H:%M:%S)Z"
