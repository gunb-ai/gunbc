#!/usr/bin/env bash
# #14892 @353133e47416 fleet-converge reset_observer fix. ONE head-built claim_batch for ALL witness arms
# (interpreter reads the TREE's .dag at runtime; deterministic CB = one binary both sides).
# Regen via per-tree gunbc (freshness gate binds binary to its build tree), unbound as CI (probe finding #14893).
# MUTANT = head tree with ONLY the roster-exclusion filter removed from reset_observer_peer_roster
# (guarded: exact expected line, single-line diff, mutant line present).
# Expect: head witness PASS + regen rc=0 yml byte-identical + writable-guard restores;
# mutant: w_the_real_peer_roster_excludes_... mint witness FAILs among others; regen rc=1 InputChoice.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver892 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver892
echo 51539607552 > /sys/fs/cgroup/ver892/memory.max
RB=$(cat /sys/fs/cgroup/ver892/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver892 memory.max=$RB (read-back, bounded; claim arms bound, regen unbound per CI shape)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14892/head:refs/verifier/f14892a || { echo FETCH_FAIL; exit 9; }
FULL=$(git rev-parse refs/verifier/f14892a)
[ "${FULL:0:12}" = "353133e47416" ] || { echo "PR_REF_MOVED got=$FULL -- STOP"; exit 9; }
echo "HEAD_VERIFIED=$FULL"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
T=/tmp/w892_h; M=/tmp/w892_m
for d in $T $M; do git worktree prune >/dev/null 2>&1; rm -rf $d; done
git worktree add -q --force --detach $T "$FULL" || { echo WT_FAIL; exit 9; }
git worktree add -q --force --detach $M "$FULL" || { echo WTM_FAIL; exit 9; }
ROSTER=dag/gunbc/host/host_reset_subject_roster.dag
EXPECT='  filter(map(fleet_hosts(), h => h as NonEmptyStr), h => host_roster_contains(roster: subjects, host: h) == false)'
got=$(sed -n '190p' $T/$ROSTER); [ "$got" = "$EXPECT" ] || { echo "MUT_LINE_GUARD_FAIL got=[$got]"; exit 9; }
awk 'NR==190{print "  map(fleet_hosts(), h => h as NonEmptyStr)"; next}{print}' $T/$ROSTER > $M/$ROSTER || exit 9
cmp -s <(sed -n '190p' $T/$ROSTER) <(sed -n '190p' $M/$ROSTER) && { echo MUT_NOOP; exit 9; }
grep -qx '  map(fleet_hosts(), h => h as NonEmptyStr)' $M/$ROSTER || { echo MUT_INCOMPLETE; exit 9; }
git -C $M diff --name-only $FULL -- . | grep -vx "$ROSTER" | grep . && { echo MUT_EXTRA_FILE; exit 9; }
git -C $M diff $FULL -- $ROSTER | grep -c '^[-+][^-+]' | sed 's/^/MUT_CHANGED_LINES=/'
diff -rq $T/dag $M/dag | wc -l | sed 's/^/DAG_DIFF_FILES=/'
echo TREES_OK
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t892cb
(cd $T && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1)
CB=/root/workspace/repo-root/.t892cb/release/claim_batch
[ -x $CB ] || { echo NO_CB; exit 9; }
echo "CB_SHA=$(sha $CB) (ONE binary, all witness arms)"
WIT=dag/test/claim/host/host_reset_return_witness.dag
MTWIT=dag/test/claim/runner/mtcollins1_runner_offer_witness.dag
runclaims() { # tag tree entry listfile chunksize wet
  local tag=$1 tree=$2 entry=$3 lf=$4 cs=$5 wet=$6
  local n=0 i=0 total; total=$(wc -l < $lf)
  echo "  [$tag $(basename $entry) total=$total] $(date -u +%H:%M:%S)Z"
  while [ $i -lt $total ]; do
    local fns out rc
    fns=$(sed -n "$((i+1)),$((i+cs))p" $lf | paste -sd, -)
    n=$((n+1)); i=$((i+cs))
    out=/tmp/w892_${tag}_${n}.out
    ( cd $tree && exec 11>/sys/fs/cgroup/ver892/cgroup.procs && echo $BASHPID >&11 && \
      timeout 1800 $CB --claim-run $wet --source-root dag --source-root src/v2 --entry $entry --functions "$fns" > $out 2>&1 )
    rc=$?
    echo "    chunk$n rc=$rc PASS=$(grep -ac '^PASS ' $out) FAIL=$(grep -ac '^FAIL ' $out) BYTES=$(wc -c < $out) $(date -u +%H:%M:%S)Z"
    grep -a '^FAIL ' $out | sed 's/^/      F| /'
    grep -aiE 'not held|panic|UNOBSERVED' $out | head -3 | sed 's/^/      E| /' | cut -c1-190
  done
}
echo "=== HEAD witness claims (bound) $(date -u +%H:%M:%S)Z"
runclaims h $T $WIT $REPO/tools/fns892_hostreset.txt 8 ""
echo "=== HEAD mtcollins claims (bound) $(date -u +%H:%M:%S)Z"
runclaims hmt $T $MTWIT $REPO/tools/fns892_mtcollins.txt 8 ""
echo "=== MUTANT witness claims (bound) $(date -u +%H:%M:%S)Z"
runclaims m $M $WIT $REPO/tools/fns892_hostreset.txt 8 ""
# ---- regen: per-tree gunbc, unbound ----
(cd $T && CARGO_TARGET_DIR=/root/workspace/repo-root/.t892gh cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GH=/root/workspace/repo-root/.t892gh/release/gunbc
[ -x $GH ] || { echo NO_GUNBC_H; exit 9; }
(cd $M && CARGO_TARGET_DIR=/root/workspace/repo-root/.t892gm cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GM=/root/workspace/repo-root/.t892gm/release/gunbc
[ -x $GM ] || { echo NO_GUNBC_M; exit 9; }
YML=.github/workflows/fleet-converge.yml
echo "GUNBC_HEAD_SHA=$(sha $GH)"
echo "GUNBC_MUT_SHA=$(sha $GM) (differ by embedded host path, Rust source identical; per-tree is the freshness-gate requirement)"
regen() { # tag tree gunbc
  local tag=$1 tree=$2 g=$3
  local out=/tmp/w892_r_$tag.out
  cd $tree || exit 9
  timeout 2400 $g run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet > $out 2>&1
  local rc=$?
  echo "  [regen $tag] rc=$rc BYTES=$(wc -c < $out) PORCELAIN=[$(git status --porcelain | tr '\n' ';')] $(date -u +%H:%M:%S)Z"
  grep -aiE 'InputChoice|refus|drift|fleet-converge' $out | head -6 | sed 's/^/    R| /' | cut -c1-200
  cd $REPO || exit 9
}
echo "=== REGEN head $(date -u +%H:%M:%S)Z"; regen h $T $GH
echo "=== REGEN mutant $(date -u +%H:%M:%S)Z"; regen m $M $GM
# writable guard: corrupt yml on head tree, regen must restore byte-identical (proves the route WRITES this artifact)
echo "== writable guard (head tree) $(date -u +%H:%M:%S)Z"
cp $T/$YML /tmp/w892_yml_orig
echo "# verifier-corruption-guard" >> $T/$YML
(cd $T && CARGO_TARGET_DIR=/root/workspace/repo-root/.t892gh timeout 2400 $GH run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet > /tmp/w892_r_guard.out 2>&1); rc=$?
echo "  [guard] rc=$rc RESTORED=$(cmp -s /tmp/w892_yml_orig $T/$YML && echo BYTE_IDENTICAL || echo DIFFERS) PORCELAIN=[$(git -C $T status --porcelain -- $YML)]"
grep -aiE 'InputChoice|refus' /tmp/w892_r_guard.out | head -3 | sed 's/^/    G| /' | cut -c1-200
echo DONE_14892A
