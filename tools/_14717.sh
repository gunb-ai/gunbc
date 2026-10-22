#!/usr/bin/env bash
# #14717 @ 8437981a6d50... (checkout-trust witnesses). Arms:
#  A. HEAD: 3 claims by name -> expect 3 PASS.
#  B. BASE a4a189d5390 (author's base, old witnesses): same 3 names -> expect 3 FAIL.
#     If a4a189d lacks the shared claim entry, fall back to merge-base; report which.
#  C. Mutation 1: boot ref expr -> needs.build.outputs.revision (fleet_converge_event_sha_checkout_ref_expr
#     body changed) => boot claim FAILs; namecheap FAILs (reads boot step); printer expected GREEN (its
#     subjects are build+apply+deploy refs, NOT boot) -- capture this explicitly.
#  D. Mutation 2: apply ref loses "|| github.sha" => both apply-reading claims (printer, namecheap) FAIL.
#  E. Restore byte-exact, re-PASS.
#  F. Static: no 'contains' in the 3 claim files; '<rev>'/'rev' not sourced from inputs; equality only.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=8437981a6d5031c6077a67672e6c1c742526569c  # gh pr view 14717 --json headRefOid (re-resolved below if guard fires)
BASE=a4a189d5390
echo "HEADSHA_expected=$OID"
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verF17 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verF17
echo 51539607552 > /sys/fs/cgroup/verF17/memory.max
RB=$(cat /sys/fs/cgroup/verF17/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verF17 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verF17/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
BOOT=dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag
PRINTER=dag/test/claim/printer_workflow_binding_witness.dag
NC=dag/test/claim/namecheap_workflow_witness.dag
FC=dag/gunbc/fleet/fleet_converge_workflow.dag
BN=witness_the_checkout_ref_is_an_exact_equality_trusting_the_branch_protecting_rule_not_a_contains_of_event_sha
PN=witness_the_workflow_pins_the_real_machine_and_checkout_of_the_converged_revision
NN=witness_the_workflow_uses_the_fleet_converge_checkout_and_secret_channel
runnames() { # wt tag
  local wt=$1 tag=$2
  timeout 900 $CB --claim-run --hermetic --source-root dag --entry $BOOT --function "$BN" > /tmp/f17_${tag}_boot.out 2>&1
  local r1=$?
  timeout 900 $CB --claim-run --hermetic --source-root dag --entry $PRINTER --function "$PN" > /tmp/f17_${tag}_printer.out 2>&1
  local r2=$?
  timeout 900 $CB --claim-run --hermetic --source-root dag --entry $NC --function "$NN" > /tmp/f17_${tag}_nc.out 2>&1
  local r3=$?
  echo "  [$tag] boot rc=$r1: $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/f17_${tag}_boot.out | head -1) | printer rc=$r2: $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/f17_${tag}_printer.out | head -1) | namecheap rc=$r3: $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/f17_${tag}_nc.out | head -1)"
  for l in boot printer nc; do
    grep -aE '^(FAIL|UNOBSERVED) ' /tmp/f17_${tag}_$l.out | head -2 | sed "s/^/    $l| /" | cut -c1-230
  done
  return 0
}
# --- A. head ---
rm -rf /tmp/wf17
git fetch -q origin +refs/pull/14717/head:refs/verifier/f17 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f17)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f17) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wf17 $OID || { echo WT_FAIL; exit 9; }
cd /tmp/wf17 || exit 9
for f in $BOOT $PRINTER $NC $FC; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
echo "STAGED_OK 4 head blobs; claim-name check:"
grep -a "test fn $BN" $BOOT >/dev/null && grep -a "test fn $PN" $PRINTER >/dev/null && grep -a "test fn $NN" $NC >/dev/null && echo "  names exist at head" || { echo "  NAME_MISSING -- names read from head:"; grep -aE 'test fn witness_the_checkout|test fn witness_the_workflow' $BOOT $PRINTER $NC; exit 8; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf17
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf17/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
echo "=== A. HEAD claims by name (expect 3 PASS):"
runnames /tmp/wf17 head
echo "=== F. statics:"
echo "  'contains' in 3 claim files: $(grep -ac contains $BOOT $PRINTER $NC | paste -sd' ')"
grep -anE 'event_sha|github\.sha|inputs|expected_revision' $BOOT $PRINTER $NC | grep -av '^\s*#' | head -6 | sed 's/^/    G| /' | cut -c1-200

# --- B. BASE witnesses, same binary ---
echo "=== B. BASE at $BASE (expect 3 FAIL with this binary):"
git -C /tmp/wf17 fetch -q origin $BASE 2>/dev/null || true
if git -C /tmp/wf17 cat-file -e ${BASE}^{} 2>/dev/null; then
  B=$(git -C /tmp/wf17 rev-parse ${BASE}^{})
else
  git -C /tmp/wf17 fetch -q origin https://github.com/gunb-ai/gunbc $BASE 2>/dev/null || true
  B=$(git -C /tmp/wf17 rev-parse ${BASE}^{} 2>/dev/null || echo "")
fi
if [ -n "$B" ]; then
  rm -rf /tmp/wb17; git -C /tmp/wf17 worktree add -q --detach /tmp/wb17 $B || echo "  BASE_WT_FAIL"
  if [ -d /tmp/wb17 ]; then
    cd /tmp/wb17
    [ "$(git cat-file blob $B:$BOOT 2>/dev/null | sha256sum | cut -d' ' -f1)" = "$(sha $BOOT)" ] || echo "  note: boot blob differs at base (old witness expected)"
    runnames /tmp/wb17 base
    cd /tmp/wf17
  fi
else
  echo "  BASE_COMMIT_UNREACHABLE -- fetching full history shallow-fallback"
  git fetch -q --unshallow origin 2>/dev/null || git fetch -q origin 2>/dev/null || true
  echo "  retry manually"
fi

# --- C. mutation 1: boot ref expr -> needs['build'].outputs.revision ---
echo "=== C. MUT1 boot ref -> needs.build.outputs (boot+namecheap must FAIL; printer capture):"
cp -f $FC /tmp/fc_head_orig
grep -an "github.sha" $FC | head -8 | sed 's/^/    FC| /' | cut -c1-200
