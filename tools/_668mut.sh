#!/usr/bin/env bash
# #14668 @ (OID below, re-guarded): suppression mutation for the two emit-route refusal claims.
# A. baseline 4 claims by name wet (2 controls + 2 refusals) expect 4 PASS
# B. MUTATION: suppress the ambiguity diagnostic -- both emit sites'
#    compile_error!("ambiguous anonymous record literal shape; add a nominal type")
#    -> () . Rebuild. The 2 refusal claims must FAIL; the 2 controls must PASS.
# C. restore byte-exact, rebuild, re-PASS 4/4.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=7c97832508ad532bcc996b5ac520b862c43cb33f  # re-set programmatically before launch
BASE=daeda4c6a92c6dfd7c3a54bf49bdb0f638983505
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver668m 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver668m
echo 51539607552 > /sys/fs/cgroup/ver668m/memory.max
RB=$(cat /sys/fs/cgroup/ver668m/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver668m memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver668m/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=dag/test/claim/record_emission_candidate_set_witness.dag
RFM=dag/gunbc/recurring_failure_mode/witnesses_red_on_main_with_no_failure_mode_row.dag
EMIT=src/v1/stage0/src/v1_compiler_emit_rust.rs
run4() { # tag
  local tag=$1
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/e668_${tag}.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/e668_${tag}.out) FAIL=$(grep -ac '^FAIL ' /tmp/e668_${tag}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/e668_${tag}.out)"
  for n in $(echo $NAMES | tr ',' ' '); do
    echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/e668_${tag}.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/e668_${tag}.out)"
  done
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/e668_${tag}.out | head -4 | sed 's/^/    V| /' | cut -c1-220
}
rm -rf /tmp/w668m
git fetch -q origin +refs/pull/14668/head:refs/verifier/668m || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/668m)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/668m) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w668m $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w668m || exit 9
for f in $WIT $RFM $EMIT; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
NAMES=$(grep -oE '^fn (w_[a-z_0-9]+)' $WIT | sed 's/^fn //' >/dev/null; grep -oE '^test fn [a-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd,)
echo "NAMES_FROM_FILE=$(echo $NAMES | tr ',' ' ')"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tb3
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tb3/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
cp -f $EMIT /tmp/emit_head_orig
echo "=== A. baseline 4 by name wet (expect 4 PASS):"
run4 base
echo "=== B. MUTATION suppress ambiguity diagnostic (both emit sites -> ()), rebuild:"
PATTERN='compile_error!(\"ambiguous anonymous record literal shape; add a nominal type\")'
BEFORE=$(grep -acF "$PATTERN" $EMIT)
echo "  anchor count before=$BEFORE (2 expected)"
[ "$BEFORE" = "2" ] || { echo "MUT_ANCHOR_FAIL -- NOT a mutation run"; }
perl -pi -e "s/\Q$PATTERN\E/()/g" $EMIT
AFTER=$(grep -acF "$PATTERN" $EMIT)
[ "$AFTER" = "0" ] || { echo "MUT_REPLACE_FAIL after=$AFTER"; }
echo "  mutated sha=$(sha $EMIT) orig=$(sha /tmp/emit_head_orig)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "  rebuild CB_SHA=$(sha $CB)"
run4 mut
echo "=== C. restore byte-exact, rebuild, re-run:"
cp -f /tmp/emit_head_orig $EMIT
[ "$(sha $EMIT)" = "$(sha /tmp/emit_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "  post CB_SHA=$(sha $CB)"
run4 post
git status --porcelain | head -3
echo "DONE_668M $(date -u +%H:%M:%S)Z"
