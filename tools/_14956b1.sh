#!/usr/bin/env bash
# #14956 dispatch B: claim arms with the ONE dispatch-A binary (PR delta = 0 .rs => data-only subject;
# sha reprinted and must equal dispatch A's).
# Arms:
#  G1 head gates --wet            enforcement_consistency_gate_holds + only_cost_contract_carries_red_control_probe => PASS,PASS exit 0
#  G2 head gates DRY (no --wet)   => UNOBSERVED exit 2 (non-vacuity: the claims really consult the live tree)
#  G3 P  (pre-deletion 115143..) gates --wet => gate FAIL (cause B real: WitnessOnly schedule_lens consumer)
#  G4 M  (Blocking emitted_binary repointed at a truly-absent file) gate => FAIL (the RED)
#  G5 V  (AuditOnly vacuity repointed at same absent file) gate => PASS (other-shape control: gate judges only Blocking)
#  G6 head probe claim only_cost_contract_carries_red_control_probe (single-fn run, same binary) => PASS
#  S  importer sweep: 14 claim files (86 test fns, matches body 61 PASS/25 FAIL), head vs P, FAIL sets compared BY NAME.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=1a2cfbedf763df63e5713d0f84d1a7b0e14dfd82
BASE_SHA=115143a4d8a1e870124e7848fa5084c3fcfe1c38
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
[ "$(git rev-parse refs/verifier/f14956b 2>/dev/null)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14956b 2>/dev/null) -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t956
H=/tmp/w956h; P=/tmp/w956p; M=/tmp/w956m; V=/tmp/w956v
for T in $H $P $M $V; do [ -d $T ] || { echo "WORKTREE_MISSING $T -- run dispatch A first"; exit 9; }; done
echo "=== build-if-missing: $(date -u +%H:%M:%S)Z"
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL; exit 9; }
CB=$CARGO_TARGET_DIR/release/claim_batch
[ -x $CB ] || { echo BUILD_MISSING; exit 9; }
echo "HEAD_BIN sha256=$(sha $CB) (must equal dispatch A)"
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
  local cg=/sys/fs/cgroup/ver956-$tag
  rm -rf $cg 2>/dev/null || true; mkdir -p $cg
  echo 51539607552 > $cg/memory.max
  local RB; RB=$(cat $cg/memory.max)
  { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEM_READBACK_FAIL $tag got=$RB"; echo "VOID $tag rc=9 (mem cgroup unbound)"; return 9; }
  local extra=""
  [ "$mode" = wet ] && extra=--wet
  ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
    timeout 2400 $CB --claim-run $extra --source-root dag --source-root src/v2 --entry $entry --functions "$fns" ) > $d/out.txt 2>&1
  local rc=$?
  echo "ROW $tag tree=$(branch_of $tree) entry=$entry mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) lines=$(wc -l < $d/out.txt)"
  grep -E '^(PASS|FAIL|UNOBSERVED) ' $d/out.txt | sed "s|^|  $tag |"
  grep -E '^(UNOBSERVED|.*runtime error|\[resolve)' $d/out.txt | head -4 | sed "s|^|  $tag note: |"
  cp $d/out.txt /tmp/956-$tag.out
}
branch_of () { case $1 in
  $H) echo head;; $P) echo base115143a;; $M) echo mutated-Blocking;; $V) echo mutated-AuditOnly;;
  *) echo other;; esac; }
echo "=== G1 head gates wet $(date -u +%H:%M:%S)Z"
run1 G1 $H $ELW wet $GATE $PROBE
echo "=== G2 head gates dry $(date -u +%H:%M:%S)Z"
run1 G2 $H $ELW dry $GATE $PROBE
echo "=== G3 pre-deletion gates $(date -u +%H:%M:%S)Z"
run1 G3 $P $ELW wet $GATE $PROBE
echo "=== G4 Blocking-repoint RED $(date -u +%H:%M:%S)Z"
run1 G4 $M $ELW wet $GATE $PROBE
echo "=== G5 AuditOnly-repoint control $(date -u +%H:%M:%S)Z"
run1 G5 $V $ELW wet $GATE $PROBE
echo "=== G6 probe alone $(date -u +%H:%M:%S)Z"
run1 G6 $H $ELW wet $PROBE

echo "DISPATCH_B1_DONE $(date -u +%H:%M:%S)Z"
