#!/usr/bin/env bash
# #14886 @a87a952f4492 FrozenPathDeferral bounded drop. ONE head-built gunbc + claim_batch (same dispatch).
# Instrument arms via `gunbc test //gunbc/instruments:witness-deferral-freeze-population`, run IN ITS BUILD
# TREE (freshness gate), unbound as CI (probe finding #14893: governor refuses inherited bound cgroup).
# Fixture knobs: GUNBC_CI_DIFF_BASE (operator override) + UNCOMMITTED roster edits (gate reads the live FS
# for the current side; checkout==HEAD^{commit} stays true while edits are uncommitted).
# ARMS: CONTROL(B=HEAD)=0 | RED plant new (entry,fn) identity =>1 names it | DUP exact-duplicate row =>0
#       (identity JOIN, not count: +1 row same identity set) | REMOVAL delete a row =>0 | NO-BASELINE(no env)=>2
# SPOT via claim_batch (bound cgroup): commit_witness_claim_roster_holds,
# roster_std_change_dag_zero_suspects_within_ratchet. SKIP_SPOTS=1 runs instrument arms only.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver886 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver886
echo 51539607552 > /sys/fs/cgroup/ver886/memory.max
RB=$(cat /sys/fs/cgroup/ver886/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver886 memory.max=$RB (read-back, bounded; claim arms bound, instrument arms unbound per CI shape)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14886/head:refs/verifier/f14886a || { echo FETCH_FAIL; exit 9; }
FULL=$(git rev-parse refs/verifier/f14886a)
[ "${FULL:0:12}" = "a87a952f4492" ] || { echo "PR_REF_MOVED got=$FULL -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$FULL"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
T=/tmp/w886_h
git worktree prune >/dev/null 2>&1; rm -rf $T
git worktree add -q --force --detach $T "$FULL" || { echo WT_FAIL; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t886
if [ -x /root/workspace/repo-root/.t886/release/gunbc ] && [ -x /root/workspace/repo-root/.t886/release/claim_batch ]; then
  echo CACHED
else
  (cd $T && cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1)
fi
GUNBC=/root/workspace/repo-root/.t886/release/gunbc
CB=/root/workspace/repo-root/.t886/release/claim_batch
[ -x $GUNBC ] && [ -x $CB ] || { echo NO_BINARY; exit 9; }
echo "GUNBC_SHA=$(sha $GUNBC)"
echo "CB_SHA=$(sha $CB)"
ROSTER=dag/gunbc/witness/witness_deferral_freeze.dag
cp $T/$ROSTER /tmp/w886_roster_orig
TARGET='//gunbc/instruments:witness-deferral-freeze-population'
restore() { cp /tmp/w886_roster_orig $T/$ROSTER; }
runinst() {
  local tag=$1; shift
  local out=/tmp/w886_i_$tag.out
  ( cd $T && env "$@" timeout 2400 $GUNBC test "$TARGET" > $out 2>&1 ); local rc=$?
  echo "  [inst $tag] rc=$rc BYTES=$(wc -c < $out) $(date -u +%H:%M:%S)Z"
  if [ "$(wc -c < $out)" -lt 600 ]; then echo "    RAWDUMP:"; sed 's/^/    R| /' $out | cut -c1-200; fi
  grep -aiE 'bounded|unbounded|NO OBSERVATION|refus|added|frozen' $out | head -8 | sed 's/^/    I| /' | cut -c1-200
}
echo "== CONTROL $(date -u +%H:%M:%S)Z"; restore; runinst control GUNBC_CI_DIFF_BASE=HEAD
# RED: one added FrozenPathDeferral row, EXISTING entry path + NOVEL function name (identity = (entry,fn) pair)
grep -rq 'verifier_planted_identity_886' $T/dag && { echo PLANT_COLLIDES; exit 9; }
python3 - <<'PY'
import re
orig=open('/tmp/w886_roster_orig').read()
m=re.search(r'^data frozen_path_deferrals: List<FrozenPathDeferral> = \[\n', orig, re.M)
assert m, "decl not found"
end=orig.index('\n];', m.end())
ins='''  FrozenPathDeferral {
    entry: "dag/test/claim/artifact_store_fs_witness.dag",
    functions: [
      "verifier_planted_identity_886_holds"
    ]
  },
'''
out=orig[:end+1] + ins + orig[end+1:]
open('/tmp/w886_p_roster','w').write(out)
a=orig.count('FrozenPathDeferral {'); b=out.count('FrozenPathDeferral {')
assert b==a+1, (a,b)
print("PLANT_ROWS %d->%d" % (a,b))
PY
[ -f /tmp/w886_p_roster ] || { echo PLANT_FILE_MISSING; exit 9; }
cp /tmp/w886_p_roster $T/$ROSTER || exit 9
echo "== RED $(date -u +%H:%M:%S)Z"; runinst red GUNBC_CI_DIFF_BASE=HEAD
# DUP: exact duplicate of the first row => +1 row, identity SET unchanged. Bounded => join, not count.
python3 - <<'PY'
import re
orig=open('/tmp/w886_roster_orig').read()
m=re.search(r'^data frozen_path_deferrals: List<FrozenPathDeferral> = \[\n', orig, re.M)
assert m
s=orig.index('  FrozenPathDeferral {', m.end())
e=orig.index('  },', s)+len('  },\n')
block=orig[s:e]
end=orig.index('\n];', m.end())
out=orig[:end+1] + block + orig[end+1:]
open('/tmp/w886_d_roster','w').write(out)
print("DUP_ROWS %d->%d block=%r" % (orig.count('FrozenPathDeferral {'), out.count('FrozenPathDeferral {'), block[:50]))
PY
[ -f /tmp/w886_d_roster ] || { echo DUP_FILE_MISSING; exit 9; }
restore
cp /tmp/w886_d_roster $T/$ROSTER || exit 9
echo "== DUP $(date -u +%H:%M:%S)Z"; runinst dup GUNBC_CI_DIFF_BASE=HEAD
# REMOVAL: delete the first row entirely (shrinking-only: must stay bounded)
python3 - <<'PY'
import re
orig=open('/tmp/w886_roster_orig').read()
m=re.search(r'^data frozen_path_deferrals: List<FrozenPathDeferral> = \[\n', orig, re.M)
assert m
s=orig.index('  FrozenPathDeferral {', m.end())
e=orig.index('  },', s)+len('  },\n')
out=orig[:s] + orig[e:]
open('/tmp/w886_rm_roster','w').write(out)
a=orig.count('FrozenPathDeferral {'); b=out.count('FrozenPathDeferral {')
assert b==a-1, (a,b)
print("REMOVAL_ROWS %d->%d removed=%r" % (a,b, orig[s:s+50]))
PY
[ -f /tmp/w886_rm_roster ] || { echo RM_FILE_MISSING; exit 9; }
restore
cp /tmp/w886_rm_roster $T/$ROSTER || exit 9
echo "== REMOVAL $(date -u +%H:%M:%S)Z"; runinst removal GUNBC_CI_DIFF_BASE=HEAD
restore
echo "== NO-BASELINE $(date -u +%H:%M:%S)Z"; runinst nobase -u GUNBC_CI_DIFF_BASE -u GITHUB_ACTIONS
git -C $T status --porcelain | head -3
spot() {
  local tag=$1 entry=$2 fn=$3
  local out=/tmp/w886_s_$tag.out
  ( cd $T && exec 11>/sys/fs/cgroup/ver886/cgroup.procs && echo $BASHPID >&11 && \
    timeout 3300 $CB --claim-run --source-root dag --source-root src/v2 --entry $entry --functions "$fn" > $out 2>&1 )
  local rc=$?
  echo "  [spot $tag] rc=$rc PASS=$(grep -ac "^PASS $fn" $out) FAIL=$(grep -ac "^FAIL $fn" $out) PANIC=$(grep -ac "^PANICKED" $out) BYTES=$(wc -c < $out) $(date -u +%H:%M:%S)Z"
  grep -aE "^(PASS|FAIL|PANICKED) $fn" $out | head -2 | sed 's/^/    S| /' | cut -c1-200
  grep -aiE 'not held|runtime error|refus' $out | head -3 | sed 's/^/    E| /' | cut -c1-190
}
if [ "${SKIP_SPOTS:-0}" = "1" ]; then
  echo SPOTS_SKIPPED
else
  echo "== SPOT commit_witness_claim_roster_holds $(date -u +%H:%M:%S)Z"
  spot cwcr dag/test/claim/long/commit_witness_claim_roster_witness.dag commit_witness_claim_roster_holds
  echo "== SPOT roster_std_change $(date -u +%H:%M:%S)Z"
  spot rsc src/v2/test/claim/complexity/accumulator_copy_roster_gate_std.dag roster_std_change_dag_zero_suspects_within_ratchet
fi
echo DONE_14886A
