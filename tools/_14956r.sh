#!/usr/bin/env bash
# #14956 job runner: ordered arms, per-arm checkpoints in /tmp/c4v/ckpt956 (survives the runner's
# ~45m deadline; re-invoke to resume). Deadline arg in seconds (default 2400).
# Usage: bash tools/_14956r.sh [deadline_s]
set -u
cd "$(dirname "$0")/.." || exit 9
DEADLINE=${1:-2400}
START=$(date +%s)
. tools/_14956prelude.sh
CKPT=/tmp/c4v/ckpt956; mkdir -p $CKPT/jobs
echo "CKPT=$CKPT deadline=${DEADLINE}s start=$(date -u +%H:%M:%S)Z"
rm -rf $H $P $M $V; git worktree prune >/dev/null 2>&1
wt $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
wt $P $BASE_SHA || { echo WTP_FAIL; exit 9; }
wt $M $HEAD_SHA || { echo WTM_FAIL; exit 9; }
wt $V $HEAD_SHA || { echo WTV_FAIL; exit 9; }
python3 tools/_14956mut.py $M $V || { echo MUTATION_FAIL -- STOP; exit 9; }
build_head || exit 9
CB=$CARGO_TARGET_DIR/release/claim_batch
ELW=src/v2/test/claim/enforcement_live_witness.dag
GATE=enforcement_consistency_gate_holds
PROBE=only_cost_contract_carries_red_control_probe
branch_of () { case $1 in
  $H) echo head;; $P) echo base115143a;; $M) echo mutated-Blocking;; $V) echo mutated-AuditOnly;;
  *) echo other;; esac; }
elapsed () { echo $(( $(date +%s) - START )); }
runjob () { # runjob <tag> <tree> <entry> <wet|dry> <fn1> [..]   (skip if checkpoint present)
  local tag=$1; shift; local tree=$1; shift; local entry=$1; shift; local mode=$1; shift
  [ -s $CKPT/jobs/$tag.out ] && { echo "ROW $tag SKIPPED-CHECKPOINT"; return 0; }
  local el; el=$(elapsed)
  if [ "$el" -gt "$DEADLINE" ]; then echo "DEADLINE_REACHED at ${el}s before $tag -- resume later"; return 1; fi
  local fns; fns=$(printf '%s,' "$@" | sed 's/,$//')
  local d=/tmp/956j-$tag; rm -rf $d; mkdir -p $d
  arm_cgroup $tag || { echo "VOID $tag rc=9 (mem cgroup)"; return 9; }
  local cg=$CGPATH extra=""
  [ "$mode" = wet ] && extra=--wet
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 2100 $CB --claim-run $extra --source-root dag --source-root src/v2 --entry $entry --functions "$fns" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag tree=$(branch_of $tree) entry=$entry mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) lines=$(wc -l < $d/out.txt) el=${el}s"
  grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |" | head -30
  grep -E 'runtime error|NO OBSERVATION|outcomes-out' $d/out.txt | head -3 | sed "s|^|  $tag note: |"
  mv $d/out.txt $CKPT/jobs/$tag.out
}
sweep_job () { # sweep_job <tag> <idx> <file> <tree>
  local tag=$1 i=$2 f=$3; local CKPT_TREE=$4
  [ -s $CKPT/jobs/SW-$tag-$i.out ] && { echo "ROW SW-$tag-$i file=$f SKIPPED-CHECKPOINT"; return 0; }
  local el; el=$(elapsed)
  if [ "$el" -gt "$DEADLINE" ]; then echo "DEADLINE_REACHED at ${el}s before SW-$tag-$i -- resume later"; return 1; fi
  local fns; fns=$(fns_of2 $CKPT_TREE/$f | paste -sd, -)
  [ -n "$fns" ] || { echo "ROW SW-$tag-$i file=$f FNS=EMPTY -- VOID (write checkpoint to keep it loud)"; printf 'VOID empty-fns %s\n' "$f" > $CKPT/jobs/SW-$tag-$i.out; return 0; }
  local n; n=$(echo $fns | tr ',' '\n' | wc -l)
  local d=/tmp/956j-sw$tag$i; rm -rf $d; mkdir -p $d
  arm_cgroup sw$tag$i || { echo "ROW SW-$tag-$i VOID (mem cgroup)"; return 9; }
  local cg=$CGPATH
  ( cd $CKPT_TREE && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 2100 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $f --functions "$fns" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW SW-$tag-$i file=$f n=$n rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) el=${el}s"
  mv $d/out.txt $CKPT/jobs/SW-$tag-$i.out
}
rc=0
echo "=== gates $(date -u +%H:%M:%S)Z"
runjob G4 $M $ELW wet $GATE $PROBE || rc=$?
runjob G5 $V $ELW wet $GATE $PROBE || rc=$?
echo "=== sweeps $(date -u +%H:%M:%S)Z"
i=0; while read -r f; do i=$((i+1)); sweep_job head $i $f $H || { rc=$?; break; }; done < tools/_14956files.txt
[ "$rc" = 0 ] || { echo "PARTIAL head=$(ls $CKPT/jobs/SW-head-*.out 2>/dev/null | wc -l)/14 — re-invoke"; exit 0; }
i=0; while read -r f; do i=$((i+1)); sweep_job base $i $f $P || { rc=$?; break; }; done < tools/_14956files.txt
[ "$rc" = 0 ] || { echo "PARTIAL base=$(ls $CKPT/jobs/SW-base-*.out 2>/dev/null | wc -l)/14 — re-invoke"; exit 0; }
echo "=== compare $(date -u +%H:%M:%S)Z"
for side in head base; do
  cat $CKPT/jobs/SW-$side-*.out | grep '^FAIL ' | awk '{print $2}' | sort > $CKPT/fail-$side.txt
  cat $CKPT/jobs/SW-$side-*.out | grep '^PASS ' | awk '{print $2}' | sort > $CKPT/pass-$side.txt
  cat $CKPT/jobs/SW-$side-*.out | grep '^UNOBSERVED ' | awk '{print $2}' | sort > $CKPT/unobs-$side.txt
  echo "SIDE $side PASS=$(wc -l < $CKPT/pass-$side.txt) FAIL=$(wc -l < $CKPT/fail-$side.txt) UNOBSERVED=$(wc -l < $CKPT/unobs-$side.txt)"
done
echo "--- FAIL-set diff (head vs base), EMPTY means identical by name:"
diff $CKPT/fail-head.txt $CKPT/fail-base.txt && echo "FAIL_SETS_IDENTICAL_BY_NAME"
echo "--- FAIL names (base):"; cat $CKPT/fail-base.txt
echo "--- UNOBSERVED (head):"; cat $CKPT/unobs-head.txt
echo "ALL_DONE $(date -u +%H:%M:%S)Z"
