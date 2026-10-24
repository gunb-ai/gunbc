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
ARMS=${2:-all}
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
CKPT=/tmp/c4v/ckpt933; mkdir -p $CKPT/jobs
echo "CKPT=$CKPT deadline=${DEADLINE}s"
H=/tmp/w933h; B=/tmp/w933b
git worktree prune >/dev/null 2>&1 || true
NEED_B=1; ls $CKPT/jobs/T1.out $CKPT/jobs/T2.out >/dev/null 2>&1 && ls $CKPT/jobs/T2b.out $CKPT/jobs/T3.out >/dev/null 2>&1 && NEED_B=0
echo "NEED_B=$NEED_B"
elapsed () { echo $(( $(date +%s) - START )); }
# ---------- head tree + build
[ -d $H ] || git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/.t933h
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
mkdir -p $H/target/release
cp -f $CARGO_TARGET_DIR/release/claim_batch $H/target/release/claim_batch
echo "BUILD_COMMIT=$HEAD_SHA"
echo "HEAD_BIN sha256=$(sha $H/target/release/claim_batch)"
# ---------- base tree: head tree minus PR diff (reverse-apply, guarded)
if [ "$NEED_B" = 1 ]; then [ -d $B ] || git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }; fi
if [ "$NEED_B" = 1 ]; then
  rm -rf $B 2>/dev/null; git worktree prune >/dev/null 2>&1
  git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
  git diff origin/main...refs/verifier/f14933 > $CKPT/pr.patch
  echo "PR_PATCH_BYTES=$(wc -c < $CKPT/pr.patch) FILES=$(grep -c '^diff --git' $CKPT/pr.patch)"
  if (cd $B && git apply -R --check $CKPT/pr.patch 2>$CKPT/apply-err.txt); then
    (cd $B && git apply -R $CKPT/pr.patch) && echo REVERSE_APPLY_OK || { echo REVERSE_APPLY_EXEC_FAIL -- STOP; exit 9; }
    (cd $B && git status --porcelain | wc -l) | grep -qv ^0 || { echo "NOOP-REVERSE -- STOP"; exit 9; }
  else
    echo REVERSE_APPLY_CHECK_FAIL:
    head -6 $CKPT/apply-err.txt | sed 's|^|  apply: |'
    echo "FALLBACK: worktree at merge-base $BASE_SHA"
    rm -rf $B; git worktree prune >/dev/null 2>&1
    git worktree add -q --force --detach $B $BASE_SHA || { echo WTB_FALLBACK_FAIL; exit 9; }
    echo BASE_AT_MERGE_BASE
  fi
  (cd $B && git status --porcelain | wc -l) | grep -qv ^0 || { echo "NOOP-REVERSE -- STOP"; exit 9; }
  touch $CKPT/reverse-applied
fi
HF=$(grep -c -- "--outcomes-out" $H/src/v1/stage0/src/bin/claim_batch.rs)
BF=$(grep -c -- "--outcomes-out" $B/src/v1/stage0/src/bin/claim_batch.rs)
echo "BASE_PROOF head-flag-lines=$HF base-flag-lines=$BF"
[ "$BF" -lt "$HF" ] || { echo BASE_NOT_OLD -- STOP; exit 9; }
export CARGO_TARGET_DIR=/root/workspace/.t933b
if [ "$NEED_B" = 1 ]; then (cd $B && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL_BASE; exit 9; }; fi
if [ "$NEED_B" = 1 ]; then mkdir -p $B/target/release; cp -f $CARGO_TARGET_DIR/release/claim_batch $B/target/release/claim_batch; fi
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
fns_of2 () { python3 "$(dirname "$0")/fns_safe.py" $1; }
runjob () { # runjob <tag> <tree> <bin> <mode wet|dry> <entry> <fns-csv> [extra args...]
  local tag=$1 tree=$2 bin=$3 mode=$4 entry=$5 fns=$6; shift 6
  if [ "$ARMS" != all ]; then case ",$ARMS," in *",$tag,"*) ;; *) echo "ROW $tag SKIPPED-ARGLIST"; return 0;; esac; fi
  [ -s $CKPT/jobs/$tag.out ] && { echo "ROW $tag (checkpoint) P=$(grep -c '^PASS ' $CKPT/jobs/$tag.out) F=$(grep -c '^FAIL ' $CKPT/jobs/$tag.out) U=$(grep -c '^UNOBSERVED ' $CKPT/jobs/$tag.out)"; grep -E "^(PASS|FAIL|UNOBSERVED) " $CKPT/jobs/$tag.out | sed "s|^|  $tag |" | head -30; return 0; }
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
  if [ $(wc -l < $d/out.txt) -le 80 ]; then sed "s|^|  $tag |" $d/out.txt; else grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |" | head -40; fi
  grep -aE "outcomes-out" $d/out.txt | head -3 | sed "s|^|  $tag |"
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
[ -x $CBB ] || { echo "BASE_BIN_MISSING — will build next pass"; rc=7; }
[ "$rc" = 0 ] && runjob T2b $B $CBB dry $COST $COSTFN || true
echo "=== T4 wire entry WITH --outcomes-out (resolve re-measure) $(date -u +%H:%M:%S)Z"
runjob T4 $H $CBH wet $WIRE "an_unrecognised_outcome_kind_refuses_located_at_the_kind" --outcomes-out /tmp/c4v/14933-ledger-wire.tsv || rc=$?
echo "=== T1b cost claim WITH flag, targeted resolve line $(date -u +%H:%M:%S)Z"
if [ -s $CKPT/jobs/T1b.out ]; then echo "ROW T1b (checkpoint)"; grep -aE "outcomes-out|^PASS " $CKPT/jobs/T1b.out | sed 's|^|  T1b |'; else
runjob T1b $H $CBH dry $COST $COSTFN --outcomes-out /tmp/c4v/14933-ledger-cost.tsv || rc=$?
fi
echo "=== T3 --outcomes-out on BASE must refuse $(date -u +%H:%M:%S)Z"
[ -x $CBB ] && runjob T3 $B $CBB dry $COST $COSTFN --outcomes-out /tmp/c4v/14933-ledger-base.tsv || true
if [ -s $CKPT/jobs/T2.out ] && [ -s $CKPT/jobs/T2b.out ]; then
  # strip the resolve/elapsed timing noise, then compare bytes
  for x in T2 T2b; do grep -vE "^\[resolve|resolve_wire_closure_ms|el=|resolved_ms" $CKPT/jobs/$x.out > $CKPT/$x.norm; done
  if diff -q $CKPT/T2.norm $CKPT/T2b.norm >/dev/null 2>&1; then echo "UNCHANGED_VS_MAIN byte-identical (timing-stripped)"; else echo "UNCHANGED_VS_MAIN DIFFERS (first 24 diff lines):"; diff $CKPT/T2.norm $CKPT/T2b.norm | head -24 | sed "s|^|  diff: |"; fi
  # verdict-only compare: PASS/FAIL/UNOBSERVED lines must match exactly
  for x in T2 T2b; do grep -E '^(PASS|FAIL|UNOBSERVED) ' $CKPT/jobs/$x.out > $CKPT/$x.verdicts; done
  diff $CKPT/T2.verdicts $CKPT/T2b.verdicts && echo "VERDICTS_IDENTICAL_vs_main"
fi
echo "ALL_DONE_14933 $(date -u +%H:%M:%S)Z"
