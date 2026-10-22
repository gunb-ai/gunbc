#!/usr/bin/env bash
# #14692 @ 779fd98addff — d2: ledger 9 (floor census) + 18 (row coherence) by name wet;
# COLLAPSE MUTATION: claim_batch.rs ClosureDiagnostics branch 1 -> 2; the closure claim (expects 1)
# must FAIL while the missing-entry claim (expects 2) still PASSes; restore byte-exact, re-PASS.
# Statics: format_error_nodes sites in entry_resolve.rs; any String-typed error path left there.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=779fd98addfff08aed3de2e269a14b393437c7a5
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver692c 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver692c
echo 51539607552 > /sys/fs/cgroup/ver692c/memory.max
RB=$(cat /sys/fs/cgroup/ver692c/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver692c memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver692c/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
FC=src/v2/test/claim/workflow/floor_terminal_ledger.dag
LC=dag/test/claim/ledger_row_coherence.dag
WIT=dag/test/claim/claim_batch_unobserved_exit_witness.dag
ER=src/v1/stage0/src/cli_run/entry_resolve.rs
CBR=src/v1/stage0/src/bin/claim_batch.rs
RED1=an_entry_with_a_closure_error_exits_1_with_a_fail_line
RED2=a_nonexistent_entry_exits_2_with_a_located_refusal
git fetch -q origin +refs/pull/14692/head:refs/verifier/692c || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/692c)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/692c) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w692c 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w692c; git worktree add --detach /tmp/w692c $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w692c ] || { echo WT_FAIL; exit 9; }
cd /tmp/w692c || exit 9
for f in $FC $LC $WIT $ER $CBR; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
FCN=$(grep -oE '^test fn [a-z_0-9]+' $FC | sed 's/^test fn //' | paste -sd,)
LCN=$(grep -oE '^test fn [a-z_0-9]+' $LC | sed 's/^test fn //' | paste -sd,)
echo "HEADSHA=$OID FC_COUNT=$(echo $FCN | tr ',' ' ' | wc -w) LC_COUNT=$(echo $LCN | tr ',' ' ' | wc -w)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t692c
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t692c/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
runf() { # entry names tag
  local entry=$1 names=$2 tag=$3
  timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $entry --functions "$names" > /tmp/s692_${tag}.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/s692_${tag}.out) FAIL=$(grep -ac '^FAIL ' /tmp/s692_${tag}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/s692_${tag}.out)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/s692_${tag}.out | head -4 | sed 's/^/    V| /' | cut -c1-230
}
echo "=== A. ledger floor census 9 by name wet:"
runf $FC "$FCN" fc9
echo "=== B. ledger row coherence 18 by name wet:"
runf $LC "$LCN" lc18
echo "=== C. collapse mutation: claim_batch.rs ClosureDiagnostics exit 1 -> 2 (rebuild):"
cp -f $CBR /tmp/cbr_head_orig
python3 - <<'PY'
p='src/v1/stage0/src/bin/claim_batch.rs'
s=open(p).read()
old='''            EntryResolveRefusal::ClosureDiagnostics { .. } => {
                println!(
                    "FAIL {} (closure diagnostics in {}: {})",
                    function,
                    group.entry,
                    refusal.message().lines().next().unwrap_or("")
                );
                timings.any_failed = true;
            }'''
new='''            EntryResolveRefusal::ClosureDiagnostics { .. } => {
                println!(
                    "FAIL {} (closure diagnostics in {}: {})",
                    function,
                    group.entry,
                    refusal.message().lines().next().unwrap_or("")
                );
                timings.any_unobserved = true;
            }'''
n=s.count(old)
print("  collapse anchor count:",n)
assert n==1, n
open(p,'w').write(s.replace(old,new))
PY
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "  mut CB_SHA=$(sha $CB)"
echo "  -- expect $RED1 FAIL (expects 1, gets 2), $RED2 PASS (expects 2 unchanged):"
runf $WIT "$RED1,$RED2" collapse
for n in $RED1 $RED2; do
  echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/s692_collapse.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/s692_collapse.out)"
done
echo "=== D. restore byte-exact + rebuild + re-PASS both:"
cp -f /tmp/cbr_head_orig $CBR
[ "$(sha $CBR)" = "$(sha /tmp/cbr_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "  post CB_SHA=$(sha $CB)"
runf $WIT "$RED1,$RED2" post
echo "=== E. statics: format_error_nodes sites + String error path in entry_resolve.rs:"
grep -acn 'format_error_nodes' $ER | sed 's/^/  format_error_nodes total mentions: /'
grep -an 'DiagnosticStringSource\|DiagnosticNamedSource' $ER | wc -l | sed 's/^/  Diagnostics-source sites: /'
grep -an 'Vec<String>\|-> String\|: String' $ER | grep -i 'error\|diagnostic\|refus' | head -5 | sed 's/^/  STRING-ERR| /' | cut -c1-190
grep -an 'panic!\|unwrap()' $ER | head -3 | sed 's/^/  PANIC-SCAN| /' | cut -c1-150
git status --porcelain | head -3
echo "DONE_14692_D2 $(date -u +%H:%M:%S)Z"
