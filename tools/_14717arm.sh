#!/usr/bin/env bash
# #14717 @ 70dd5a9b88e4 — ONE arm per dispatch (each claim run ~8min: 2621-module resolve).
# ARM=m1   boot-ref-to-needs: expect ALL 3 FAIL
# ARM=m2   shared-job-if := always(): expect printer+namecheap FAIL, boot PASS
# ARM=m3   PrinterWorkflow => shared id: expect printer FAIL, namecheap PASS
# ARM=base a4a189d5390 worktree, same binary+names: expect ALL 3 FAIL
set -u
ARM=${ARM:?need ARM}
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO ARM=$ARM"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=70dd5a9b88e4de60a04ffc7bcbf64257ffc38ca5
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver17$ARM 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver17$ARM
echo 51539607552 > /sys/fs/cgroup/ver17$ARM/memory.max
RB=$(cat /sys/fs/cgroup/ver17$ARM/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver17$ARM memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver17$ARM/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
BOOT=dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag
PRINTER=dag/test/claim/printer_workflow_binding_witness.dag
NC=dag/test/claim/namecheap_workflow_witness.dag
FC=dag/gunbc/fleet/fleet_converge_workflow.dag
BN=another_expected_revision_cannot_change_a_byte_the_boot_job_executes
PN=trusted_code_cannot_be_replaced_by_dispatch_revision
run1() { # dir entry fn tag
  local dir=$1 entry=$2 fn=$3 tag=$4
  cd $dir || { echo "  [$tag] DIR_FAIL"; return; }
  timeout 1200 $CB --claim-run --hermetic --source-root dag --source-root src/v2 --entry $entry --function "$fn" > /tmp/g17_${tag}.out 2>&1
  local rc=$?
  local v=$(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/g17_${tag}.out | head -1)
  echo "  [$tag] rc=$rc ${v:-NO-VERDICT}"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/g17_${tag}.out | head -2 | sed 's/^/    V| /' | cut -c1-230
  grep -a 'witnesses)\|running 1 witness' /tmp/g17_${tag}.out | head -1 | sed 's/^/    W| /' | cut -c1-140
}
restore() { cp -f /tmp/fc_head_orig $FC; [ "$(sha $FC)" = "$(sha /tmp/fc_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"; }
git fetch -q --unshallow origin 2>/dev/null || true
git worktree remove --force /tmp/wg17 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wg17; git worktree add --detach /tmp/wg17 $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/wg17 ] || { echo WT_FAIL; exit 9; }
cd /tmp/wg17 || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "WT_NOT_HEAD"; exit 9; }
for f in $BOOT $PRINTER $NC $FC; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf17
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf17/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
cp -f $FC /tmp/fc_head_orig
case $ARM in
 m1)
  python3 - <<'PY'
p='dag/gunbc/fleet/fleet_converge_workflow.dag'
s=open(p).read()
old='data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_event_sha)\n'
new='data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_checkout_ref)\n'
n=s.count(old)
print("  m1 anchor count:",n)
assert n==1, n
open(p,'w').write(s.replace(old,new))
PY
  A=$(grep -ac 'data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_checkout_ref)' $FC)
  [ "$A" = "1" ] || { echo "MUT_ANCHOR_FAIL got=$A -- NOT a mutation run"; }
  echo "  mutated sha=$(sha $FC)"
  run1 /tmp/wg17 $BOOT $BN m1_boot; run1 /tmp/wg17 $PRINTER $PN m1_printer; run1 /tmp/wg17 $NC $PN m1_nc
  restore ;;
 m2)
  sed -i 's/^data fleet_converge_shared_job_if: String = fleet_converge_any_mode_step_if(modes: filter(fleet_converge_workflow_modes, mode => fleet_converge_mode_job_id(mode: mode) == fleet_converge_shared_job_id))$/data fleet_converge_shared_job_if: String = "always()"/' $FC
  A=$(grep -ac 'data fleet_converge_shared_job_if: String = "always()"' $FC)
  [ "$A" = "1" ] || { echo "MUT_ANCHOR_FAIL got=$A -- NOT a mutation run"; }
  echo "  mutated sha=$(sha $FC)"
  run1 /tmp/wg17 $BOOT $BN m2_boot; run1 /tmp/wg17 $PRINTER $PN m2_printer; run1 /tmp/wg17 $NC $PN m2_nc
  restore ;;
 m3)
  sed -i 's/^    PrinterWorkflow => "printer" as NonEmptyStr$/    PrinterWorkflow => fleet_converge_shared_job_id/' $FC
  A=$(grep -ac '    PrinterWorkflow => fleet_converge_shared_job_id' $FC)
  [ "$A" = "1" ] || { echo "MUT_ANCHOR_FAIL got=$A -- NOT a mutation run"; }
  echo "  mutated sha=$(sha $FC)"
  run1 /tmp/wg17 $PRINTER $PN m3_printer; run1 /tmp/wg17 $NC $PN m3_nc; run1 /tmp/wg17 $BOOT $BN m3_boot
  restore ;;
 base)
  B=$(git rev-parse a4a189d5390^{} 2>/dev/null || true)
  if [ -z "$B" ]; then git fetch -q origin a4a189d5390 2>/dev/null || true; B=$(git rev-parse a4a189d5390^{} 2>/dev/null || true); fi
  [ -n "$B" ] || { echo BASE_UNREACHABLE; exit 9; }
  echo "  base=$B"
  git worktree remove --force /tmp/bg17 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
  rm -rf /tmp/bg17; git worktree add --detach /tmp/bg17 $B 2>&1 | tail -2 | sed 's/^/  WT| /'
  [ -d /tmp/bg17 ] || { echo BASE_WT_FAIL; exit 9; }
  run1 /tmp/bg17 $BOOT $BN base_boot; run1 /tmp/bg17 $PRINTER $PN base_printer; run1 /tmp/bg17 $NC $PN base_nc ;;
 *) echo "bad ARM=$ARM"; exit 9 ;;
esac
git status --porcelain | head -3
echo "DONE_14717_ARM=$ARM $(date -u +%H:%M:%S)Z"
