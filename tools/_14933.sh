#!/usr/bin/env bash
# #14933 runner: ONE exact-head claim_batch (a61ef827c9c6), copied into the head tree so the WET
# witnesses' child spawn (root/target/release/claim_batch) is the same binary.
# Arms (all wet where the module declares SubstrateInputsOnly; wire claims run wet too):
#  W  floor_terminal_ledger_wire: ALL test fns incl. the unknown-kind RED and held controls  (18 per body)
#  O  claim_batch_outcomes_out_witness: planted code-2 -> UNOBSERVED row rc semantics; held control PASS
#  S  preemption_reachability: w_unobserved_reachability_is_never_admitted + siblings (13 fns)
#  T1 dag-only --outcomes-out cost_entry.dag --function cost_entry_holds  (resolve+render, ledger head-only)
#  T2 same WITHOUT --outcomes-out (unchanged-vs-main control, run at base tree too)
# Base tree = head with the PR's own diff reverse-applied (subject is BOTH data and driver: cli_run/claim_batch
# are .rs => two binaries: head-built and base-built; each runs its OWN tree; cross-run impossible).
set -u
cd "$(dirname "$0")/.." || exit 9
DEADLINE=${1:-2400}
START=$(date +%s)
HEAD_SHA=a61ef827c9c6fe6c049d04e99cca412522af3d91
BASE_SHA=$(git merge-base origin/main refs/verifier/f14933 2>/dev/null || echo "")
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
git fetch -q origin +refs/pull/14933/head:refs/verifier/f14933 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14933)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14933) -- STOP"; exit 9; }
git cat-file -e origin/main^{commit} 2>/dev/null || git fetch -q origin main || true
[ -n "$BASE_SHA" ] || BASE_SHA=$(git merge-base origin/main refs/verifier/f14933)
echo "BASE_SHA=$BASE_SHA (merge-base main..head; base tree = head - reverse(PR diff))"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
H=/tmp/w933h; B=/tmp/w933b
git worktree prune >/dev/null 2>&1 || true
elapsed () { echo $(( $(date +%s) - START )); }
CKPT=/tmp/c4v/ckpt933; mkdir -p $CKPT/jobs
echo "CKPT=$CKPT deadline=${DEADLINE}s"
# ---------- head tree + build
[ -d $H ] || git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
CARGO_TARGET_DIR=$H/.t933
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
mkdir -p $H/target/release
cp -f $CARGO_TARGET_DIR/release/claim_batch $H/target/release/claim_batch
echo "BUILD_COMMIT=$HEAD_SHA"
echo "HEAD_BIN sha256=$(sha $H/target/release/claim_batch)"
# ---------- base tree: head tree minus PR diff (reverse-apply, guarded)
[ -d $B ] || git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
if [ ! -s $CKPT/reverse-applied ]; then
  git diff origin/main...refs/verifier/f14933 > $CKPT/pr.patch
  (cd $B && git apply -R --check $CKPT/pr.patch && git apply -R $CKPT/pr.patch && echo REVERSE_APPLY_OK) || { echo "REVERSE_APPLY_FAIL -- STOP"; exit 9; }
  # guard: reverse-apply must actually change the subject files
  (cd $B && git status --porcelain | wc -l) | grep -qv '^0$' || { echo "NOOP-REVERSE -- STOP"; exit 9; }
  touch $CKPT/reverse-applied
fi
CARGO_TARGET_DIR=$B/.t933b
(cd $B && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL_BASE; exit 9; }
mkdir -p $B/target/release
cp -f $CARGO_TARGET_DIR/release/claim_batch $B/target/release/claim_batch
echo "BASE_BIN sha256=$(sha $B/target/release/claim_batch)"
CBH=$H/target/release/claim_batch
CBB=$B/target/release/claim_batch
arm_cgroup () {
  local cg=/sys/fs/cgroup/ver933-$1
  rm -rf $cg 2>/dev/null || true; mkdir -p $cg || return 9
  echo 51539607552 > $cg/memory.max || return 9
  local RB; RB=$(cat $cg/memory.max)
  { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEM_READBACK_FAIL $1 got=$RB"; return 9; }
  CGPATH=$cg
}
fns_of2 () { perl -0777 -ne 'while(/test fn\s+([A-Za-z0-9_]+)\s*\(/g){print "$1\n"}' $1 | sort -u; }
runjob () { # runjob <tag> <tree> <bin> <mode wet|dry> <entry> <fns-csv> [extra args...]
  local tag=$1 tree=$2 bin=$3 mode=$4 entry=$5 fns=$6; shift 6
  [ -s $CKPT/jobs/$tag.out ] && { echo "ROW $tag SKIPPED-CHECKPOINT"; return 0; }
  local el; el=$(elapsed)
  [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before $tag"; return 1; }
  local d=/tmp/933j-$tag; rm -rf $d; mkdir -p $d
  arm_cgroup $tag || { echo "VOID $tag (mem)"; return 9; }
  local cg=$CGPATH extra=""
  [ "$mode" = wet ] && extra=--wet
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 2400 $bin --claim-run $extra --source-root dag --source-root src/v2 --entry $entry --functions "$fns" "$@" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag tree=$([ $tree = $H ] && echo head || echo base) mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) el=${el}s"
  grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |" | head -40
  grep -E '\[outcomes-out\]|outcomes-out\]' $d/out.txt | head -3 | sed "s|^|  $tag |"
  mv $d/out.txt $CKPT/jobs/$tag.out
}
rc=0
WIRE=src/v2/test/claim/workflow/floor_terminal_ledger_wire.dag
OUTW=dag/test/claim/claim_batch_outcomes_out_witness.dag
PREM=dag/test/claim/preemption_reachability.dag
echo "=== W wire (head) $(date -u +%H:%M:%S)Z"
W=$(fns_of2 $H/$WIRE | paste -sd, -)
runjob W $H $CBH wet $WIRE "$W" || rc=$?
echo "=== O outcomes-out witnesses (head) $(date -u +%H:%M:%S)Z"
O=$(fns_of2 $H/$OUTW | paste -sd, -)
runjob O $H $CBH wet $OUTW "$O" || rc=$?
echo "=== S preemption (head) $(date -u +%H:%M:%S)Z"
S=$(fns_of2 $H/$PREM | paste -sd, -)
runjob S $H $CBH wet $PREM "$S" || rc=$?
COST=dag/test/claim/witness_row_cost_projection.dag
COSTFN=witness_row_cost_projection_holds
echo "=== T1 dag+src/v2 --outcomes-out head $(date -u +%H:%M:%S)Z"
runjob T1 $H $CBH dry $COST $COSTFN --outcomes-out /tmp/c4v/14933-ledger-head.tsv || rc=$?
echo "=== T2 dag+src/v2 NO --outcomes-out head $(date -u +%H:%M:%S)Z"
runjob T2 $H $CBH dry $COST $COSTFN || rc=$?
echo "=== T2b dag+src/v2 NO --outcomes-out BASE $(date -u +%H:%M:%S)Z"
runjob T2b $B $CBB dry $COST $COSTFN || rc=$?
echo "=== T3 --outcomes-out on BASE must refuse $(date -u +%H:%M:%S)Z"
runjob T3 $B $CBB dry $COST $COSTFN --outcomes-out /tmp/c4v/14933-ledger-base.tsv || rc=$?
echo "ALL_DONE_14933 $(date -u +%H:%M:%S)Z"
