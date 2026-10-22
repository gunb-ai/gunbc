#!/usr/bin/env bash
# #14719 @ 73a3b4dd59004416984c8e0e070a2f79ffb2f87d (wet-schedule uniqueness in the .dag fold).
# Arms: baseline both entries by name; mutation (composition call site scheduled: [] ) expect RED
# claim FAILs with siblings PASS; byte-exact restore + re-PASS. Control eval_steps + schedule rows.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
[ "$(git rev-parse origin/session/nimble-badger-800 2>/dev/null)" != "$(git rev-parse HEAD 2>/dev/null)" ] && git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=73a3b4dd59004416984c8e0e070a2f79ffb2f87d  # gh pr view 14719 --json headRefOid
echo "HEADSHA_expected=$OID"
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verF19 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verF19
echo 51539607552 > /sys/fs/cgroup/verF19/memory.max
RB=$(cat /sys/fs/cgroup/verF19/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verF19 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verF19/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
TST=src/v2/test/wet_evidence.dag
CLM=src/v2/test/claim/local_repo_wet_terminal.dag
WFE=src/v2/workflow/wet_evidence.dag
rm -rf /tmp/wf19
git fetch -q origin +refs/pull/14719/head:refs/verifier/f19 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f19)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f19) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wf19 $OID || { echo WT_FAIL; exit 9; }
cd /tmp/wf19 || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "WT_NOT_AT_HEAD"; exit 9; }
for f in $TST $CLM $WFE; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
echo "STAGED_OK 3 head blobs verified"
# 8 exhaustive matches carrying the new arm
echo "ARM_COUNT_IN_TEST=$(grep -ac 'WetScheduledIdentityDuplicated' $TST) (8 expected: cause_tag + six is_* + finalization join)"
echo "ANCHOR_COUNT=$(grep -ac 'right: wet_scheduled_identity_duplicate_causes(scheduled: scheduled)' $WFE) (1 expected)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf19
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf19/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
cp -f $WFE /tmp/wfe_head_orig
RED=a_scheduled_identity_written_twice_refuses_naming_it
SIB1=missing_unscheduled_and_duplicated_stay_three_distinct_causes
SIB2=the_route_is_derived_and_a_foreign_route_terminal_refuses
SIB3=an_unmet_host_premise_refuses_as_the_runner_and_a_met_premise_failure_as_the_verdict
SIB4=a_wrong_entry_and_a_wrong_function_stay_two_distinct_causes
CTL=the_real_schedule_has_no_duplicated_identity
CTL2=the_local_roster_is_not_empty
run2() { # tag
  local tag=$1
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $TST --functions "$RED,$SIB1,$SIB2,$SIB3,$SIB4" > /tmp/f19_${tag}_test.out 2>&1
  local rc1=$?
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $CLM --functions "$CTL,$CTL2" > /tmp/f19_${tag}_clm.out 2>&1
  local rc2=$?
  echo "  [$tag] test-entry rc=$rc1 PASS=$(grep -ac '^PASS ' /tmp/f19_${tag}_test.out) FAIL=$(grep -ac '^FAIL ' /tmp/f19_${tag}_test.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f19_${tag}_test.out) | clm-entry rc=$rc2 PASS=$(grep -ac '^PASS ' /tmp/f19_${tag}_clm.out) FAIL=$(grep -ac '^FAIL ' /tmp/f19_${tag}_clm.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f19_${tag}_clm.out)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/f19_${tag}_test.out /tmp/f19_${tag}_clm.out | head -6 | sed 's/^/    V| /' | cut -c1-220
  for f in $RED $SIB1 $SIB2 $SIB3 $SIB4; do
    echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/f19_${tag}_test.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/f19_${tag}_test.out)"
  done
  for f in $CTL $CTL2; do
    echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/f19_${tag}_clm.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/f19_${tag}_clm.out)"
  done
  grep -a 'eval_steps' /tmp/f19_${tag}_clm.out | grep -a "$CTL" | sed 's/^/    W| /' | cut -c1-200
}
echo "=== A. baseline at head: $(date -u +%H:%M:%S)Z"
run2 base
echo "=== B. mutation: composition call site scheduled: scheduled -> scheduled: []"
sed -i 's/right: wet_scheduled_identity_duplicate_causes(scheduled: scheduled)/right: wet_scheduled_identity_duplicate_causes(scheduled: [])/' $WFE
MUT_ANCHOR=$(grep -ac 'scheduled: \[\])' $WFE)
[ "$MUT_ANCHOR" = "1" ] || { echo "MUT_ANCHOR_FAIL got=$MUT_ANCHOR -- NOT a mutation run"; }
echo "  mutated sha=$(sha $WFE) orig_sha=$(sha /tmp/wfe_head_orig)"
run2 mut
echo "=== C. restore byte-exact + re-run RED"
cp -f /tmp/wfe_head_orig $WFE
[ "$(sha $WFE)" = "$(sha /tmp/wfe_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"
run2 post
git status --porcelain | head -3
echo "DONE_14719 $(date -u +%H:%M:%S)Z"
