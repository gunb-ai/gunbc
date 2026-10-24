#!/usr/bin/env bash
# #14956 dispatch B (phases: b1 gates | b2 sweep-head | b3 sweep-base+compare)
# ONE dispatch-A binary (0 .rs in the PR delta => data-only subject). sha reprinted; must equal A.
set -u
cd "$(dirname "$0")/.." || exit 9
PHASE=${1:-b1}
. tools/_14956prelude.sh
rm -rf $H $P $M $V; git worktree prune >/dev/null 2>&1
wt $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
wt $P $BASE_SHA || { echo WTP_FAIL; exit 9; }
if [ "$PHASE" = b1 ]; then
  wt $M $HEAD_SHA || { echo WTM_FAIL; exit 9; }
  wt $V $HEAD_SHA || { echo WTV_FAIL; exit 9; }
  python3 tools/_14956mut.py $M $V || { echo MUTATION_FAIL -- STOP; exit 9; }
fi
build_head || exit 9
CB=$CARGO_TARGET_DIR/release/claim_batch
CKPT=/tmp/c4v/ckpt956; mkdir -p $CKPT/gates
echo "CKPT=$CKPT"
ELW=src/v2/test/claim/enforcement_live_witness.dag
GATE=enforcement_consistency_gate_holds
PROBE=only_cost_contract_carries_red_control_probe
branch_of () { case $1 in
  $H) echo head;; $P) echo base115143a;; $M) echo mutated-Blocking;; $V) echo mutated-AuditOnly;;
  *) echo other;; esac; }
run1 () { # run1 <tag> <tree> <entry> <wet|dry> <fn1> [fn2..]
  local tag=$1; shift; local tree=$1; shift; local entry=$1; shift; local mode=$1; shift
  local fns; fns=$(printf '%s,' "$@" | sed 's/,$//')
  local d=/tmp/956-$tag; rm -rf $d; mkdir -p $d
  arm_cgroup $tag || { echo "VOID $tag rc=9 (mem cgroup)"; return 9; }
  local cg=$CGPATH extra=""
  [ "$mode" = wet ] && extra=--wet
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 2400 $CB --claim-run $extra --source-root dag --source-root src/v2 --entry $entry --functions "$fns" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag tree=$(branch_of $tree) entry=$entry mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) lines=$(wc -l < $d/out.txt)"
  grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |" | head -20
  grep -E 'runtime error|\[resolve|cannot|refus' $d/out.txt | head -4 | sed "s|^|  $tag note: |"
  mkdir -p $CKPT/gates; cp $d/out.txt $CKPT/gates/$tag.out
}
sweep () { # sweep <tag> <tree>
  local tag=$1 tree=$2 i=0
  while read -r f; do
    i=$((i+1))
    mkdir -p $CKPT/$tag
    if [ -s $CKPT/$tag/$i.out ]; then
      echo "ROW SWEEP-$tag-$i file=$f SKIPPED-CHECKPOINT P=$(grep -c '^PASS ' $CKPT/$tag/$i.out) F=$(grep -c '^FAIL ' $CKPT/$tag/$i.out) U=$(grep -c '^UNOBSERVED ' $CKPT/$tag/$i.out)"
      continue
    fi
    local fns; fns=$(fns_of2 $tree/$f | paste -sd, -)
    [ -n "$fns" ] || { echo "ROW SWEEP-$tag-$i file=$f FNS=EMPTY -- VOID"; continue; }
    local n; n=$(echo $fns | tr ',' '\n' | wc -l)
    local d=/tmp/956-sw-$tag-$i; rm -rf $d; mkdir -p $d
    arm_cgroup sw$tag$i || { echo "ROW SWEEP-$tag-$i file=$f VOID (mem cgroup)"; continue; }
    local cg=$CGPATH
    ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
      timeout 1800 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $f --functions "$fns" ) > $d/out.txt 2>&1
    local rc=$?
    echo "ROW SWEEP-$tag-$i file=$f n=$n rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt)"
    mv $d/out.txt $CKPT/$tag/$i.out
  done < tools/_14956files.txt
}
case "$PHASE" in
b1)
  echo "=== G1 head gates wet $(date -u +%H:%M:%S)Z"; run1 G1 $H $ELW wet $GATE $PROBE
  echo "=== G2 head gates dry $(date -u +%H:%M:%S)Z"; run1 G2 $H $ELW dry $GATE $PROBE
  echo "=== G3 pre-deletion gates $(date -u +%H:%M:%S)Z"; run1 G3 $P $ELW wet $GATE $PROBE
  echo "=== G4 Blocking-repoint RED $(date -u +%H:%M:%S)Z"; run1 G4 $M $ELW wet $GATE $PROBE
  echo "=== G5 AuditOnly-repoint control $(date -u +%H:%M:%S)Z"; run1 G5 $V $ELW wet $GATE $PROBE
  echo "=== G6 probe alone $(date -u +%H:%M:%S)Z"; run1 G6 $H $ELW wet $PROBE
  ;;
b4)
  wt $M $HEAD_SHA || { echo WTM_FAIL; exit 9; }
  python3 tools/_14956mut.py $M $V || { echo MUTATION_FAIL -- STOP; exit 9; }
  echo "=== G4 Blocking-repoint RED $(date -u +%H:%M:%S)Z"; run1 G4 $M $ELW wet $GATE $PROBE ;;
b5)
  wt $V $HEAD_SHA || { echo WTV_FAIL; exit 9; }
  python3 tools/_14956mut.py $M $V || { echo MUTATION_FAIL -- STOP; exit 9; }
  echo "=== G5 AuditOnly-repoint control $(date -u +%H:%M:%S)Z"; run1 G5 $V $ELW wet $GATE $PROBE ;;
b2) echo "=== SWEEP head $(date -u +%H:%M:%S)Z"; sweep head $H
  echo "files-done=$(ls $CKPT/head 2>/dev/null | wc -l)/14" ;;
b3)
  echo "=== SWEEP base $(date -u +%H:%M:%S)Z"; sweep base $P
  n=$(ls $CKPT/base 2>/dev/null | wc -l); m=$(ls $CKPT/head 2>/dev/null | wc -l)
  { [ "$n" = 14 ] && [ "$m" = 14 ]; } || { echo "INCOMPLETE_SWEEP base=$n head=$m of 14 -- re-run phases"; exit 8; }
  echo "=== SET COMPARE $(date -u +%H:%M:%S)Z"
  for side in head base; do
    cat $CKPT/$side/*.out 2>/dev/null | grep '^FAIL ' | awk '{print $2}' | sort > /tmp/956-fail-$side.txt
    cat $CKPT/$side/*.out 2>/dev/null | grep '^PASS ' | awk '{print $2}' | sort > /tmp/956-pass-$side.txt
    cat $CKPT/$side/*.out 2>/dev/null | grep '^UNOBSERVED ' | awk '{print $2}' | sort > /tmp/956-unobs-$side.txt
    echo "SIDE $side PASS=$(wc -l < /tmp/956-pass-$side.txt) FAIL=$(wc -l < /tmp/956-fail-$side.txt) UNOBSERVED=$(wc -l < /tmp/956-unobs-$side.txt)"
  done
  echo "--- FAIL-set diff (head vs base), EMPTY means identical by name:"
  diff /tmp/956-fail-head.txt /tmp/956-fail-base.txt && echo "FAIL_SETS_IDENTICAL_BY_NAME"
  echo "--- FAIL names (base):"; cat /tmp/956-fail-base.txt
  echo "--- FAIL names (head):"; cat /tmp/956-fail-head.txt
  echo "--- UNOBSERVED (head):"; cat /tmp/956-unobs-head.txt
  ;;
esac
echo "DISPATCH_$PHASE_DONE $(date -u +%H:%M:%S)Z"
