#!/usr/bin/env bash
# #14717 @ 70dd5a9b88e4 (head moved: review 84288 pinned shared-job mode set by equality).
# Claims (read from head files at run time):
#   boot:     another_expected_revision_cannot_change_a_byte_the_boot_job_executes
#             (dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag)
#   printer:  trusted_code_cannot_be_replaced_by_dispatch_revision (printer_workflow_binding_witness.dag)
#   namecheap: trusted_code_cannot_be_replaced_by_dispatch_revision (namecheap_workflow_witness.dag)
# Arms: A head 3/3 PASS (hermetic + wet). B base a4a189d5390 same names, expect FAIL x3.
# M1 boot-to-needs: event_sha_checkout_ref_expr data switched to fleet_converge_checkout_ref -> all 3 fail.
# M2 if=always(): shared_job_if data := "always()" -> printer+namecheap fail, boot PASS.
# M3 shared-id=printer: mode_job_id PrinterWorkflow arm -> shared id -> printer fails only.
# Restore byte-exact after each. F statics: no 'contains'-based ref trust; refs mention no inputs.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=70dd5a9b88e4de60a04ffc7bcbf64257ffc38ca5   # gh pr view 14717 --json headRefOid (programmatic)
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
TRUST=dag/test/claim/fleet/fleet_converge_checkout_trust.dag
BN=another_expected_revision_cannot_change_a_byte_the_boot_job_executes
PN=trusted_code_cannot_be_replaced_by_dispatch_revision
run3() { # tag mode(wet|hermetic) [dir]
  local tag=$1 mode=$2 dir=${3:-/tmp/wf17}
  cd $dir || { echo "  [$tag] DIR_FAIL"; return; }
  local flag=--hermetic; [ "$mode" = wet ] && flag=--wet
  timeout 900 $CB --claim-run $flag --source-root dag --entry $BOOT --function "$BN" > /tmp/f17_${tag}.out 2>&1
  local r1=$?
  timeout 900 $CB --claim-run $flag --source-root dag --entry $PRINTER --function "$PN" > /tmp/f17_${tag}_p.out 2>&1
  local r2=$?
  timeout 900 $CB --claim-run $flag --source-root dag --entry $NC --function "$PN" > /tmp/f17_${tag}_n.out 2>&1
  local r3=$?
  echo "  [$tag $mode] boot rc=$r1 $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/f17_${tag}.out | head -1) | printer rc=$r2 $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/f17_${tag}_p.out | head -1) | namecheap rc=$r3 $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/f17_${tag}_n.out | head -1)"
  for l in "" _p _n; do
    echo "    ----- raw /tmp/f17_${tag}$l.out ($(wc -c < /tmp/f17_${tag}$l.out)B):"
    head -12 /tmp/f17_${tag}$l.out | sed "s/^/    O| /" | cut -c1-220
  done
}
restore() { cp -f /tmp/fc_head_orig $FC; [ "$(sha $FC)" = "$(sha /tmp/fc_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"; }
# --- head worktree ---
rm -rf /tmp/wf17
git fetch -q origin +refs/pull/14717/head:refs/verifier/f17 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f17)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f17) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wf17 $OID || { echo WT_FAIL; exit 9; }
cd /tmp/wf17 || exit 9
for f in $BOOT $PRINTER $NC $FC $TRUST; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
grep -a "test fn $BN" $BOOT >/dev/null && grep -a "test fn $PN" $PRINTER >/dev/null && grep -a "test fn $PN" $NC >/dev/null && echo "STAGED_OK 5 blobs + names" || { echo NAMES_MISSING; exit 8; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf17
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf17/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
cp -f $FC /tmp/fc_head_orig
echo "=== A. HEAD (expect 3 PASS):"
run3 head_h hermetic
run3 head_w wet
echo "=== F. statics:"
echo "  claim files 'contains' count: $(grep -ac contains $BOOT $PRINTER $NC | paste -sd' ')"
echo "  refs mention inputs? trust predicate asserts !contains(inputs) on checkout+build refs (lines 28-29); grep dispatch-input syntax in claim files:"
grep -naE 'inputs\[|github\.event\.inputs|workflow_dispatch' $BOOT $PRINTER $NC $TRUST | head -4 | sed 's/^/    G| /' | cut -c1-180
# --- B. BASE ---
echo "=== B. BASE $BASE same names (expect 3 FAIL):"
git -C /tmp/wf17 fetch -q origin $BASE 2>/dev/null || true
B=$(git -C /tmp/wf17 rev-parse ${BASE}^{} 2>/dev/null || echo "")
if [ -z "$B" ]; then git fetch -q origin https://github.com/gunb-ai/gunbc $BASE 2>/dev/null || true; B=$(git rev-parse ${BASE}^{} 2>/dev/null || echo ""); fi
if [ -n "$B" ]; then
  rm -rf /tmp/wb17; git -C /tmp/wf17 worktree add -q --detach /tmp/wb17 $B || echo "  BASE_WT_FAIL"
  [ -d /tmp/wb17 ] && run3 base_h hermetic /tmp/wb17
else
  echo "  BASE_COMMIT_UNREACHABLE"
fi
# --- M1 boot-to-needs: event-sha ref data switched ---
echo "=== M1 boot-to-needs (event_sha_checkout_ref_expr -> checkout_ref): expect ALL 3 FAIL:"
sed -i 's/data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_event_sha)/data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_checkout_ref)/' $FC
A=$(grep -ac 'data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_checkout_ref)' $FC)
[ "$A" = "1" ] || { echo "M1_ANCHOR_FAIL got=$A -- NOT a mutation run"; }
run3 m1_h hermetic
restore
# --- M2 shared job if := always() ---
echo "=== M2 if=always() (expect printer+namecheap FAIL, boot PASS):"
sed -i 's/data fleet_converge_shared_job_if: String = fleet_converge_any_mode_step_if(modes: filter(fleet_converge_workflow_modes, mode => fleet_converge_mode_job_id(mode: mode) == fleet_converge_shared_job_id))/data fleet_converge_shared_job_if: String = "always()"/' $FC
A=$(grep -ac 'data fleet_converge_shared_job_if: String = "always()"' $FC)
[ "$A" = "1" ] || { echo "M2_ANCHOR_FAIL got=$A -- NOT a mutation run"; }
run3 m2_h hermetic
restore
# --- M3 printer mode joins shared job ---
echo "=== M3 shared-id=printer (expect printer FAIL only):"
sed -i 's/    PrinterWorkflow => "printer" as NonEmptyStr/    PrinterWorkflow => fleet_converge_shared_job_id/' $FC
A=$(grep -ac '    PrinterWorkflow => fleet_converge_shared_job_id' $FC)
[ "$A" = "1" ] || { echo "M3_ANCHOR_FAIL got=$A -- NOT a mutation run"; }
run3 m3_h hermetic
restore
run3 post_h hermetic
git status --porcelain | head -3
echo "DONE_14717 $(date -u +%H:%M:%S)Z"
