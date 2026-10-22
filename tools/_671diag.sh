#!/usr/bin/env bash
# #14671 diagnosis: 18/18 ledger_row_coherence claims FAIL at fe0cb832829 (bat3b), passed at
# cd7af89 (bat2). Claim+RFM blobs identical head vs main. Isolate: full untruncated diagnostic
# at head; same binary on a daeda4c6 (head's main parent) worktree; print the RFM :6:7 diagnostic.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=fe0cb8328290137f2051890cdec36ccb985c12d0   # re-resolved below
BASE=daeda4c6a92c6dfd7c3a54bf49bdb0f638983505  # head's main-side parent
echo "HEADSHA_expected=$OID"
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver671d 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver671d
echo 51539607552 > /sys/fs/cgroup/ver671d/memory.max
RB=$(cat /sys/fs/cgroup/ver671d/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver671d memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver671d/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
CLM=dag/test/claim/ledger_row_coherence.dag
RFM=dag/gunbc/recurring_failure_mode/witnesses_red_on_main_with_no_failure_mode_row.dag
git fetch -q origin +refs/pull/14671/head:refs/verifier/671d || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/671d)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/671d) want=$OID -- STOP"; exit 9; }
rm -rf /tmp/w671d; git worktree add -q --detach /tmp/w671d $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w671d || exit 9
for f in $CLM $RFM; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f"; exit 9; }
done
echo "STAGED_OK"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t671d
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t671d/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
NAMES=$(grep -oE '^fn [a-z_0-9]+' $CLM | sed 's/^fn //' | paste -sd,)
echo "NAMES_FROM_FILE=$(echo $NAMES | tr ',' ' ' | wc -w)"
echo "=== A. ALL 18 by name at HEAD, wet, ENROLLED route:"
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $CLM --functions "$NAMES" > /tmp/d671_head.out 2>&1
echo "  rc=$? PASS=$(grep -ac '^PASS ' /tmp/d671_head.out) FAIL=$(grep -ac '^FAIL ' /tmp/d671_head.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/d671_head.out)"
echo "  --- first FAIL full diagnostic (untruncated, 40L):"
grep -an 'closure diagnostics' /tmp/d671_head.out | head -1
sed -n "$(grep -an 'closure diagnostics' /tmp/d671_head.out | head -1 | cut -d: -f1),+20p" /tmp/d671_head.out | sed 's/^/  D| /'
echo "=== B. RFM file :6:7 context at head:"
sed -n '1,12p' $RFM | cat -n | sed 's/^/  R| /'
echo "=== C. same binary, same two files, at main parent $BASE (is it main-side?):"
rm -rf /tmp/b671d; git worktree add -q --detach /tmp/b671d $BASE || echo "  BASE_WT_FAIL"
if [ -d /tmp/b671d ]; then
  cd /tmp/b671d
  for f in $CLM $RFM; do
    [ "$(git cat-file blob $BASE:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || echo "  note: $f differs on base"
  done
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $CLM --functions "$NAMES" > /tmp/d671_base.out 2>&1
  echo "  base rc=$? PASS=$(grep -ac '^PASS ' /tmp/d671_base.out) FAIL=$(grep -ac '^FAIL ' /tmp/d671_base.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/d671_base.out)"
  grep -an 'closure diagnostics' /tmp/d671_base.out | head -1
  sed -n "$(grep -an 'closure diagnostics' /tmp/d671_base.out | head -1 | cut -d: -f1),+8p" /tmp/d671_base.out 2>/dev/null | sed 's/^/  D| /'
  cd /tmp/w671d
fi
git status --porcelain | head -3
echo "DONE_671D $(date -u +%H:%M:%S)Z"
