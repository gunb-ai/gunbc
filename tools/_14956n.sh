#!/usr/bin/env bash
# #14956 @e4ff91bce2c (new head; old head 1a2cfbedf76 is its ancestor). Manager go: G1, G4, G5,
# importer sweep at this head; G2/G3 carried as old-head results; rfm-rung-census resolves the updated row.
# ONE head-built claim_batch runs every tree (PR delta origin/main...head: .dag only for the subject).
# gunbc built from the same tree for the census (runs from ITS OWN build tree, UNBOUND).
# Usage: bash tools/_14956n.sh <deadline_s> <arm-list|all>
set -u
cd "$(dirname "$0")/.." || exit 9
DEADLINE=${1:-2300}
ARMS=${2:-all}
START=$(date +%s)
HEAD_SHA=e4ff91bce2c967b3101c0445a0cf6ec015434570
BASE_SHA=115143a4d8a1e870124e7848fa5084c3fcfe1c38
CKPT=/tmp/c4v/ckpt956n; mkdir -p $CKPT/jobs
echo "CKPT=$CKPT deadline=${DEADLINE}s ARMS=$ARMS"
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
git fetch -q origin +refs/pull/14956/head:refs/verifier/f14956n || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14956n)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14956n) -- STOP"; exit 9; }
git cat-file -e origin/main^{commit} 2>/dev/null || git fetch -q origin main || true
git cat-file -e $BASE_SHA^{commit} 2>/dev/null || git fetch -q origin main || true
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/.t956n
H=/tmp/w956nh; P=/tmp/w956np; M=/tmp/w956nm; V=/tmp/w956nv
git worktree prune >/dev/null 2>&1 || true
elapsed () { echo $(( $(date +%s) - START )); }
want () { # want <tag> -> 0 if should run
  if [ "$ARMS" = all ]; then return 0; fi
  case ",$ARMS," in *",$1,"*) return 0;; *) return 1;; esac
}
for T in $H $P $M $V; do rm -rf $T 2>/dev/null; done
git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --force --detach $P $HEAD_SHA || { echo WTP_FAIL; exit 9; }
# P = head with the PR's OWN delta reverse-applied (keeps main identical on both sides; isolates the deletion)
git diff origin/main...refs/verifier/f14956n > $CKPT/pr14956.patch
echo "PR14956_PATCH_BYTES=$(wc -c < $CKPT/pr14956.patch) FILES=$(grep -c '^diff --git' $CKPT/pr14956.patch)"
if (cd $P && git apply -R --check $CKPT/pr14956.patch 2>$CKPT/rev-err.txt); then
  (cd $P && git apply -R $CKPT/pr14956.patch) && echo REVERSE_APPLY_OK || { echo REVERSE_APPLY_EXEC_FAIL -- STOP; exit 9; }
else
  echo "REVERSE_APPLY_CHECK_FAIL:"; head -6 $CKPT/rev-err.txt | sed 's|^|  apply: |'
  echo "FALLBACK: worktree at pre-merge $BASE_SHA"
  rm -rf $P; git worktree prune >/dev/null 2>&1
  git worktree add -q --force --detach $P $BASE_SHA || { echo WTP_FALLBACK_FAIL; exit 9; }
  echo BASE_AT_PREMERGE
fi
# base proof: deleted module + its witness restored, registry has ScheduleLens arm
[ -f $P/src/v2/lens/schedule_lens.dag ] && echo "BASE_PROOF module-restored" || { echo "BASE_NOT_OLD module-missing -- STOP"; exit 9; }
[ -f $P/src/v2/test/claim/walk_plan_schedule_lens_test.dag ] && echo "BASE_PROOF witness-restored" || { echo "BASE_NOT_OLD witness-missing -- STOP"; exit 9; }
grep -q "subject: ScheduleLens" $P/src/v2/lens/enforcement/gate.dag && echo "BASE_PROOF registry-arm-restored" || echo "BASE_NOTE no registry arm at base (pre-#14887 form)"
grep -q "lens_contract_schedule_lens" $P/src/v2/lens/enforcement/contract.dag && echo "BASE_PROOF contract-restored" || { echo "BASE_NOT_OLD contract-missing -- STOP"; exit 9; }
git worktree add -q --force --detach $M $HEAD_SHA || { echo WTM_FAIL; exit 9; }
git worktree add -q --force --detach $V $HEAD_SHA || { echo WTV_FAIL; exit 9; }
python3 tools/_14956mut.py $M $V || { echo MUTATION_FAIL -- STOP; exit 9; }
echo "=== build claim_batch + gunbc at $HEAD_SHA $(date -u +%H:%M:%S)Z"
(cd $H && cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
CB=$CARGO_TARGET_DIR/release/claim_batch; GB=$CARGO_TARGET_DIR/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo BUILD_MISSING; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA"
echo "HEAD_BIN claim_batch sha256=$(sha $CB)"
echo "HEAD_BIN gunbc sha256=$(sha $GB)"
ELW=src/v2/test/claim/enforcement_live_witness.dag
GATE=enforcement_consistency_gate_holds
PROBE=only_cost_contract_carries_red_control_probe
branch_of () { case $1 in
  $H) echo head;; $P) echo base115143a;; $M) echo mutated-Blocking;; $V) echo mutated-AuditOnly;;
  *) echo other;; esac; }
arm_cgroup () {
  local cg=/sys/fs/cgroup/ver956n-$1
  rm -rf $cg 2>/dev/null || true; mkdir -p $cg || return 9
  echo 51539607552 > $cg/memory.max || return 9
  local RB; RB=$(cat $cg/memory.max)
  { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEM_READBACK_FAIL $1 got=$RB"; return 9; }
  CGPATH=$cg
}
fns_of2 () { python3 "$(dirname "$0")/fns_safe.py" $1; }
run1 () { # run1 <tag> <tree> <entry> <wet|dry> <fns...>
  local tag=$1; shift; local tree=$1; shift; local entry=$1; shift; local mode=$1; shift
  want $tag || { echo "ROW $tag SKIPPED-ARGLIST"; return 0; }
  if [ -s $CKPT/jobs/$tag.out ]; then echo "ROW $tag (checkpoint) rc? P=$(grep -c '^PASS ' $CKPT/jobs/$tag.out) F=$(grep -c '^FAIL ' $CKPT/jobs/$tag.out) U=$(grep -c '^UNOBSERVED ' $CKPT/jobs/$tag.out)"; grep -E '^(PASS|FAIL|UNOBSERVED) ' $CKPT/jobs/$tag.out | sed "s|^|  $tag |" | head -12; return 0; fi
  local el; el=$(elapsed)
  [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before $tag"; return 1; }
  local fns; fns=$(printf '%s,' "$@" | sed 's/,$//')
  local d=/tmp/956n-$tag; rm -rf $d; mkdir -p $d
  arm_cgroup $tag || { echo "VOID $tag rc=9 (mem cgroup)"; return 9; }
  local cg=$CGPATH extra=""
  [ "$mode" = wet ] && extra=--wet
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 2100 $CB --claim-run $extra --source-root dag --source-root src/v2 --entry $entry --functions "$fns" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag tree=$(branch_of $tree) mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) el=${el}s"
  if [ $(wc -l < $d/out.txt) -le 80 ]; then sed "s|^|  $tag |" $d/out.txt; else grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |" | head -30; fi
  grep -E 'last_builtin|reach=' $d/out.txt | tail -2 | sed "s|^|  $tag |"
  mv $d/out.txt $CKPT/jobs/$tag.out
}
sweep () { # sweep <tag>
  local tag=$1; shift
  local i=0
  while read -r f; do
    i=$((i+1))
    want SW-$tag-$i || { echo "ROW SW-$tag-$i SKIPPED-ARGLIST"; continue; }
    if [ -s $CKPT/jobs/SW-$tag-$i.out ]; then echo "ROW SW-$tag-$i file=$f (checkpoint) P=$(grep -c '^PASS ' $CKPT/jobs/SW-$tag-$i.out) F=$(grep -c '^FAIL ' $CKPT/jobs/SW-$tag-$i.out) U=$(grep -c '^UNOBSERVED ' $CKPT/jobs/SW-$tag-$i.out)"; continue; fi
    local tree=$1
    local el; el=$(elapsed)
    [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before SW-$tag-$i"; return 1; }
    local fns; fns=$(fns_of2 $tree/$f | paste -sd, -)
    [ -n "$fns" ] || { echo "ROW SW-$tag-$i file=$f FNS=EMPTY -- VOID"; printf 'VOID empty-fns %s\n' "$f" > $CKPT/jobs/SW-$tag-$i.out; continue; }
    local n; n=$(echo $fns | tr ',' '\n' | wc -l)
    local d=/tmp/956n-sw$tag$i; rm -rf $d; mkdir -p $d
    arm_cgroup sw$tag$i || { echo "ROW SW-$tag-$i VOID (mem cgroup)"; return 9; }
    local cg=$CGPATH
    ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
      timeout 2000 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $f --functions "$fns" ) > $d/out.txt 2>&1
    local rc=$?
    echo "ROW SW-$tag-$i file=$f n=$n rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) el=${el}s"
    grep -E '^(FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  SW-$tag-$i |" | head -20
    mv $d/out.txt $CKPT/jobs/SW-$tag-$i.out
  done < tools/_14956files.txt
}
census () { # census: gunbc test //gunbc/instruments:rfm-rung-census from ITS OWN build tree, UNBOUND
  want CENSUS || { echo "ROW CENSUS SKIPPED-ARGLIST"; return 0; }
  if [ -s $CKPT/jobs/CENSUS.out ]; then echo "ROW CENSUS (checkpoint)"; grep -aE 'census_read|census_below|refus|a_frozen' $CKPT/jobs/CENSUS.out | head -8 | sed 's|^|  CENSUS |'; return 0; fi
  local el; el=$(elapsed)
  [ "$el" -gt "$DEADLINE" ] && { echo "DEADLINE_REACHED ${el}s before CENSUS"; return 1; }
  ( cd $H && timeout 2400 $GB test //gunbc/instruments:rfm-rung-census ) > $CKPT/jobs/CENSUS.out 2>&1
  local rc=$?
  echo "ROW CENSUS rc=$rc lines=$(wc -l < $CKPT/jobs/CENSUS.out) el=${el}s"
  grep -aE 'census_read|census_below|case |refus|REFUSED|a_frozen' $CKPT/jobs/CENSUS.out | head -12 | sed 's|^|  CENSUS |' | cut -c1-190
}
rc=0
want G1n && { echo "=== G1n gates wet $(date -u +%H:%M:%S)Z"; run1 G1n $H $ELW wet $GATE $PROBE || rc=$?; } || echo "ROW G1n SKIPPED-ARGLIST"
want G4n && { echo "=== G4n Blocking-repoint RED $(date -u +%H:%M:%S)Z"; run1 G4n $M $ELW wet $GATE $PROBE || rc=$?; } || echo "ROW G4n SKIPPED-ARGLIST"
want G5n && { echo "=== G5n AuditOnly-repoint control $(date -u +%H:%M:%S)Z"; run1 G5n $V $ELW wet $GATE $PROBE || rc=$?; } || echo "ROW G5n SKIPPED-ARGLIST"
want CENSUS && { echo "=== rfm census $(date -u +%H:%M:%S)Z"; census || rc=$?; } || echo "ROW CENSUS SKIPPED-ARGLIST"
echo "=== sweep head $(date -u +%H:%M:%S)Z"; sweep head $H || rc=$?
echo "=== sweep base $(date -u +%H:%M:%S)Z"; sweep base $P || rc=$?
if [ $(ls $CKPT/jobs/SW-head-*.out 2>/dev/null | wc -l) = 14 ] && [ $(ls $CKPT/jobs/SW-base-*.out 2>/dev/null | wc -l) = 14 ]; then
  echo "=== compare $(date -u +%H:%M:%S)Z"
  for side in head base; do
    cat $CKPT/jobs/SW-$side-*.out | grep '^FAIL ' | awk '{print $2}' | sort > $CKPT/fail-$side.txt
    cat $CKPT/jobs/SW-$side-*.out | grep '^PASS ' | awk '{print $2}' | sort > $CKPT/pass-$side.txt
    cat $CKPT/jobs/SW-$side-*.out | grep '^UNOBSERVED ' | awk '{print $2}' | sort > $CKPT/unobs-$side.txt
    echo "SIDE $side PASS=$(wc -l < $CKPT/pass-$side.txt) FAIL=$(wc -l < $CKPT/fail-$side.txt) UNOBSERVED=$(wc -l < $CKPT/unobs-$side.txt)"
  done
  diff $CKPT/fail-head.txt $CKPT/fail-base.txt && echo "FAIL_SETS_IDENTICAL_BY_NAME"
fi
echo "PARTIAL head=$(ls $CKPT/jobs/SW-head-*.out 2>/dev/null | wc -l)/14 base=$(ls $CKPT/jobs/SW-base-*.out 2>/dev/null | wc -l)/14 gate=$(ls $CKPT/jobs/G*.out 2>/dev/null | wc -l)/3"
echo "ALL_DONE_14956N $(date -u +%H:%M:%S)Z"
