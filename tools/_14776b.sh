#!/usr/bin/env bash
# #14776 @ HEAD_OID (wet-schedule function-declared check, stacked on #14719).
# Arms: baseline 6 test + 3 controls by name; MUT1 membership-off flips exactly the new RED;
# KIND plant FnItem->DataItem (RED stays PASS = fold ignores kind = kind-free answer);
# RELPATH plant (same DeclFact, wrong rel_path; RED FAILs = the fact IS observed);
# CENSUS-EMPTY on the control claim (CTL1 FAILs = non-vacuity both directions);
# regen probe on the fold path at head AND at base 3da8211 (pre-existing resolve-failed?);
# gate health via DESIGN.md route; byte-exact restore + post re-PASS.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=3e8ece450491bd7c3c86473cf3cc4740ed27248e
BASEOID=644069c197591c09215c8e0122a495036421df8c  # new base: main
[ ${#BASEOID} -eq 40 ] || { echo "BASEOID_NOT_FULL_LEN=${#BASEOID} -- STOP"; exit 9; }
[ ${#OID} -eq 40 ] || { echo "OID_NOT_FULL_LEN=${#OID} -- STOP"; exit 9; }
echo "HEADSHA_expected=$OID BASE=$BASEOID"
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver776b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver776b
echo 51539607552 > /sys/fs/cgroup/ver776b/memory.max
RB=$(cat /sys/fs/cgroup/ver776b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver776b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver776b/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
WFE=src/v2/workflow/wet_evidence.dag
TST=src/v2/test/wet_evidence.dag
CLM=src/v2/test/claim/local_repo_wet_terminal.dag
rm -rf /tmp/wf776bb
git fetch -q origin +refs/pull/14776/head:refs/verifier/f776b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f776b)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f776b) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wf776bb $OID || { echo WT_FAIL; exit 9; }
cd /tmp/wf776bb || exit 9
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "WT_NOT_AT_HEAD"; exit 9; }
for f in $WFE $TST $CLM; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
echo "STAGED_OK 3 head blobs verified"
for f in $WFE $TST $CLM; do
  a=$(git cat-file blob 3e8ece450491bd7c3c86473cf3cc4740ed27248e:$f 2>/dev/null | sha256sum | cut -d' ' -f1)
  b=$(sha $f)
  echo "  identity_vs_d2fbb8a $f: $([ "$a" = "$b" ] && echo SAME || echo CHANGED)"
done
# static head-tree facts
echo "SCHEDULE_ROWS=$(grep -ac 'WetScheduledClaim {' src/v2/workflow/local_repo_wet_terminal.dag) (body claims 515)"
echo "ARMS_WetScheduledFunctionNotDeclared_in_test=$(grep -ac 'WetScheduledFunctionNotDeclared' $TST) (8 expected)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tf776b
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tf776b/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.tf776b/release/gunbc
echo "GB_SHA=$(sha $GB) built $(date -u +%H:%M:%S)Z"
cp -f $WFE /tmp/wfe776b_orig; cp -f $TST /tmp/tst776b_orig; cp -f $CLM /tmp/clm776b_orig
echo "ORIG_SHA wfe=$(sha /tmp/wfe776b_orig) tst=$(sha /tmp/tst776b_orig) clm=$(sha /tmp/clm776b_orig)"
RED=a_scheduled_function_not_declared_by_its_entry_refuses_naming_it
RED2=a_scheduled_identity_written_twice_refuses_naming_it
SIB1=missing_unscheduled_and_duplicated_stay_three_distinct_causes
SIB2=the_route_is_derived_and_a_foreign_route_terminal_refuses
SIB3=an_unmet_host_premise_refuses_as_the_runner_and_a_met_premise_failure_as_the_verdict
SIB4=a_wrong_entry_and_a_wrong_function_stay_two_distinct_causes
CTL1=the_real_schedule_declares_every_scheduled_function
CTL2=the_real_schedule_has_no_duplicated_identity
CTL3=the_local_roster_is_not_empty
run2() {
  local tag=$1
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $TST --functions "$RED,$RED2,$SIB1,$SIB2,$SIB3,$SIB4" > /tmp/f776b_${tag}_test.out 2>&1
  local rc1=$?
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $CLM --functions "$CTL1,$CTL2,$CTL3" > /tmp/f776b_${tag}_clm.out 2>&1
  local rc2=$?
  echo "  [$tag] test-entry rc=$rc1 PASS=$(grep -ac '^PASS ' /tmp/f776b_${tag}_test.out) FAIL=$(grep -ac '^FAIL ' /tmp/f776b_${tag}_test.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f776b_${tag}_test.out) | clm-entry rc=$rc2 PASS=$(grep -ac '^PASS ' /tmp/f776b_${tag}_clm.out) FAIL=$(grep -ac '^FAIL ' /tmp/f776b_${tag}_clm.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f776b_${tag}_clm.out)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/f776b_${tag}_test.out /tmp/f776b_${tag}_clm.out | head -8 | sed 's/^/    V| /' | cut -c1-220
  for f in $RED $RED2 $SIB1 $SIB2 $SIB3 $SIB4; do
    echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/f776b_${tag}_test.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/f776b_${tag}_test.out)"
  done
  for f in $CTL1 $CTL2 $CTL3; do
    echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/f776b_${tag}_clm.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/f776b_${tag}_clm.out)"
  done
  grep -a 'eval_steps' /tmp/f776b_${tag}_clm.out | grep -a "the_real_schedule" | sed 's/^/    W| /' | cut -c1-200
}
echo "=== A. baseline at head: $(date -u +%H:%M:%S)Z"
run2 base
echo "=== B. MUT1 membership-off: fold line 'if !map_contains_key(m: membership, key: key)' -> 'if false && !map_contains_key...'  $(date -u +%H:%M:%S)Z"
A1=$(grep -ac 'if !map_contains_key(m: membership, key: key)' $WFE)
[ "$A1" = "1" ] || echo "MUT1_ANCHOR_UNEXPECTED got=$A1 -- NOT a mutation run"
sed -i 's/if !map_contains_key(m: membership, key: key)/if false \&\& !map_contains_key(m: membership, key: key)/' $WFE
echo "  mutated wfe sha=$(sha $WFE)"
run2 mut1
echo "=== C. restore wfe; KIND plant: alpha_declared kind: FnItem -> DataItem"
cp -f /tmp/wfe776b_orig $WFE
[ "$(sha $WFE)" = "$(sha /tmp/wfe776b_orig)" ] && echo "  WFE_RESTORE=YES" || { echo "WFE_RESTORE=NO -- STOP"; exit 9; }
A2=$(sed -n '/^fn alpha_declared/,/^}/p' $TST | grep -ac 'kind: FnItem')
[ "$A2" = "1" ] || echo "KIND_ANCHOR_UNEXPECTED got=$A2"
python3 - <<'PY'
import re
p='src/v2/test/wet_evidence.dag'
s=open(p).read()
m=re.search(r'(fn alpha_declared\(\) -> List<DeclFact> \{\n(?:.|\n)*?\n\})', s)
blk=m.group(1)
nb=blk.replace('kind: FnItem','kind: DataItem')
s=s.replace(blk,nb)
open(p,'w').write(s)
print("  kind plant applied:", 'kind: DataItem' in open(p).read())
PY
run2 kind
echo "=== D. restore tst; RELPATH plant: alpha_declared rel_path -> other_test.dag"
cp -f /tmp/tst776b_orig $TST
[ "$(sha $TST)" = "$(sha /tmp/tst776b_orig)" ] && echo "  TST_RESTORE=YES" || { echo "TST_RESTORE=NO -- STOP"; exit 9; }
python3 - <<'PY'
import re
p='src/v2/test/wet_evidence.dag'
s=open(p).read()
m=re.search(r'(fn alpha_declared\(\) -> List<DeclFact> \{\n(?:.|\n)*?\n\})', s)
blk=m.group(1)
nb=blk.replace('rel_path: "dag/test/claim/fixture_test.dag"','rel_path: "dag/test/claim/other_test.dag"')
assert nb!=blk, "RELPATH anchor miss"
s=s.replace(blk,nb)
open(p,'w').write(s)
print("  relpath plant applied:", 'other_test.dag' in open(p).read())
PY
run2 relpath
echo "=== E. restore tst; CENSUS-EMPTY on CTL1: decl_facts(pool_roots: [\"dag/test/claim\"]) -> []"
cp -f /tmp/tst776b_orig $TST
[ "$(sha $TST)" = "$(sha /tmp/tst776b_orig)" ] && echo "  TST_RESTORE2=YES" || { echo "TST_RESTORE2=NO -- STOP"; exit 9; }
A3=$(grep -ac 'decl_facts(pool_roots: \["dag/test/claim"\])' $CLM)
[ "$A3" = "1" ] || echo "CENSUS_ANCHOR_UNEXPECTED got=$A3"
sed -i 's/decl_facts(pool_roots: \["dag\/test\/claim"\])/decl_facts(pool_roots: [])/' $CLM
run2 census0
echo "=== F. restore clm byte-exact + post baseline"
cp -f /tmp/clm776b_orig $CLM
OK1=$([ "$(sha $WFE)" = "$(sha /tmp/wfe776b_orig)" ] && echo Y || echo N)
OK2=$([ "$(sha $TST)" = "$(sha /tmp/tst776b_orig)" ] && echo Y || echo N)
OK3=$([ "$(sha $CLM)" = "$(sha /tmp/clm776b_orig)" ] && echo Y || echo N)
echo "  RESTORE wfe=$OK1 tst=$OK2 clm=$OK3"
[ "$OK1$OK2$OK3" = "YYY" ] || { echo "RESTORE_INCOMPLETE -- post run is not at head"; }
run2 post
git status --porcelain | head -3
echo "=== G. regen probe: gate main_wet_one on the fold path, head vs base  $(date -u +%H:%M:%S)Z"
GATE=dag/gunbc/instruments/generated_artifact_gate.dag
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$WFE > /tmp/f776b_regen_fold.out 2>&1; echo "[regen-fold@head] rc=$?"; grep -aE 'refus|resolve|error|Write' /tmp/f776b_regen_fold.out | tail -3 | cut -c1-200 | sed 's/^/  | /'
echo "  fold MIRROR-CHANGED: $([ "$(sha $WFE)" = "$(sha /tmp/wfe776b_orig)" ] && echo NO || echo YES)"
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=DESIGN.md > /tmp/f776b_regen_design.out 2>&1; echo "[regen-DESIGN@head] rc=$? (gate health)"; grep -aE 'refus|resolve|error|Write' /tmp/f776b_regen_design.out | tail -3 | cut -c1-200 | sed 's/^/  | /'
git worktree add -q --detach /tmp/wf776bbb $BASEOID 2>/dev/null || { echo BASE_WT_FAIL; }
cd /tmp/wf776bbb && git stash -u -q >/dev/null 2>&1 || true
[ "$(git rev-parse HEAD)" = "$BASEOID" ] || echo "BASE_WT_NOT_AT_BASE"
timeout 600 $GB run --source-root dag --source-root src/v2 --entry $GATE --function main_wet_one --arg path=$WFE > /tmp/f776b_regen_fold_base.out 2>&1; echo "[regen-fold@base] rc=$?"; grep -aE 'refus|resolve|error|Write' /tmp/f776b_regen_fold_base.out | tail -3 | cut -c1-200 | sed 's/^/  | /'
cd /tmp/wf776bb
echo "DONE_14776 $(date -u +%H:%M:%S)Z"
