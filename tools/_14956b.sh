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
  echo "ROW $tag tree=$($branch_of $tree) entry=$entry mode=$mode rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt) lines=$(wc -l < $d/out.txt)"
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
# ---------- importer sweep ----------
cat > /tmp/956-files.txt <<'EOF'
dag/test/claim/declaration_fork_census_witness.dag
dag/test/claim/documentary_refs_witness.dag
dag/test/claim/long/decl_ref_resolution_witness.dag
dag/test/claim/v1_complexity_decl_classification_witness.dag
src/v2/test/claim/advisories/fatal_first_readers.dag
src/v2/test/claim/complexity_contract_subject_partition_witness.dag
src/v2/test/claim/construction_justification/outside_modeled_guarantee_witness.dag
src/v2/test/claim/enforcement/lens_module_gate_witness.dag
src/v2/test/claim/lens/registry/completeness.dag
src/v2/test/claim/long/grammar_coverage_fatal_route.dag
src/v2/test/lens_common/derived_equality_backfill.dag
src/v2/test/lens_cost/p9_llvm_instruction_cost_registry_owner.dag
src/v2/test/lens_enforcement/gate.dag
src/v2/test/lens_registry/sg_claims.dag
EOF
fns_of2 () {
  perl -0777 -ne 'while(/test fn\s+([A-Za-z0-9_]+)\s*\(/g){print "$1\n"}' $1 | sort -u
}
sweep () { # sweep <tag> <tree>
  local tag=$1 tree=$2 i=0
  while read -r f; do
    i=$((i+1))
    local fns; fns=$(fns_of2 $tree/$f | paste -sd, -)
    [ -n "$fns" ] || { echo "ROW SWEEP-$tag-$i file=$f FNS=EMPTY -- VOID"; continue; }
    local n; n=$(echo $fns | tr ',' '\n' | wc -l)
    local d=/tmp/956-sw-$tag-$i; rm -rf $d; mkdir -p $d
    local cg=/sys/fs/cgroup/ver956-sw$tag$i
    rm -rf $cg 2>/dev/null || true; mkdir -p $cg
    echo 51539607552 > $cg/memory.max
    local RB; RB=$(cat $cg/memory.max)
    { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "ROW SWEEP-$tag-$i file=$f VOID (mem unbound)"; continue; }
    ( cd $tree && exec 11>$cg/cgroup.procs && echo $BASHPID >&11 && \
      timeout 1800 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $f --functions "$fns" ) > $d/out.txt 2>&1
    local rc=$?
    echo "ROW SWEEP-$tag-$i file=$f n=$n rc=$rc P=$(grep -c '^PASS ' $d/out.txt) F=$(grep -c '^FAIL ' $d/out.txt) U=$(grep -c '^UNOBSERVED ' $d/out.txt)"
    mv $d/out.txt /tmp/956-sw-$tag-$i.out
  done < /tmp/956-files.txt
}
echo "=== SWEEP head $(date -u +%H:%M:%S)Z"
sweep head $H
echo "=== SWEEP base $(date -u +%H:%M:%S)Z"
sweep base $P
echo "=== SET COMPARE $(date -u +%H:%M:%S)Z"
for side in head base; do
  cat /tmp/956-sw-$side-*.out 2>/dev/null | grep '^FAIL ' | awk '{print $2}' | sort > /tmp/956-fail-$side.txt
  cat /tmp/956-sw-$side-*.out 2>/dev/null | grep '^PASS ' | awk '{print $2}' | sort > /tmp/956-pass-$side.txt
  cat /tmp/956-sw-$side-*.out 2>/dev/null | grep '^UNOBSERVED ' | awk '{print $2}' | sort > /tmp/956-unobs-$side.txt
  echo "SIDE $side PASS=$(wc -l < /tmp/956-pass-$side.txt) FAIL=$(wc -l < /tmp/956-fail-$side.txt) UNOBSERVED=$(wc -l < /tmp/956-unobs-$side.txt)"
done
echo "--- FAIL-set diff (head vs base), EMPTY means identical by name:"
diff /tmp/956-fail-head.txt /tmp/956-fail-base.txt && echo "FAIL_SETS_IDENTICAL_BY_NAME"
echo "--- FAIL names (base):"; cat /tmp/956-fail-base.txt
echo "DISPATCH_B_DONE $(date -u +%H:%M:%S)Z"
