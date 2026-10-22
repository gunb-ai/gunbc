#!/usr/bin/env bash
# #14666 @ 5a4840b9d70d — verify: (a) 18 ledger_row_coherence claims PASS wet under claim_batch;
# (b) gunbc compile of the same entry no longer reports the TWO ambiguity errors
#     (DeclarationRef-vs-RustItemDeclarationRef, GcpPolicy-vs-GcsBucketPolicyWire);
# (c) the remaining url_file_acquisition tail-position error is separate (record its presence).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=5a4840b9d70d3c33904236ec27eb12f6097dcc69
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver666 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver666
echo 51539607552 > /sys/fs/cgroup/ver666/memory.max
RB=$(cat /sys/fs/cgroup/ver666/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver666 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver666/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
LC=dag/test/claim/ledger_row_coherence.dag
G1=dag/extdeps/realization/reconcile_in_process.dag
G2=dag/extdeps/cloud/gcp/storage.dag
git fetch -q origin +refs/pull/14666/head:refs/verifier/666 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/666)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/666) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w666 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w666; git worktree add --detach /tmp/w666 $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w666 ] || { echo WT_FAIL; exit 9; }
cd /tmp/w666 || exit 9
for f in $LC $G1 $G2; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
grep -aq 'authority: DeclarationRef' $G1 && echo "STAGED_OK G1 names DeclarationRef" || echo "  note: G1 pattern not found verbatim"
grep -aq 'GcpPolicy' $G2 && echo "STAGED_OK G2 names GcpPolicy" || echo "  note: G2 pattern not found verbatim"
LCN=$(grep -oE '^test fn [A-Za-z_0-9]+' $LC | sed 's/^test fn //' | paste -sd,)
echo "HEADSHA=$OID LC_COUNT=$(echo $LCN | tr ',' ' ' | wc -w)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t666
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t666/release/claim_batch
GB=/root/workspace/repo-root/.t666/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) GB_SHA=$(sha $GB) built $(date -u +%H:%M:%S)Z"
echo "=== A. 18 ledger claims by name wet:"
timeout 2400 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $LC --functions "$LCN" > /tmp/u666_lc.out 2>&1
echo "  rc=$? PASS=$(grep -ac '^PASS ' /tmp/u666_lc.out) FAIL=$(grep -ac '^FAIL ' /tmp/u666_lc.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/u666_lc.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/u666_lc.out | head -4 | sed 's/^/    V| /' | cut -c1-220
echo "=== B. gunbc compile of the same entry; ambiguity scan:"
timeout 2400 $GB compile --entry $LC --source-root dag --source-root src/v2 > /tmp/u666_c1.out 2>&1
echo "  rc=$? bytes=$(wc -c < /tmp/u666_c1.out)"
echo "  DeclarationRef-ambiguity lines: $(grep -aciE 'ambiguous.*(DeclarationRef|RustItemDeclarationRef)' /tmp/u666_c1.out)"
echo "  GcpPolicy-ambiguity lines:      $(grep -aciE 'ambiguous.*(GcpPolicy|GcsBucketPolicyWire)' /tmp/u666_c1.out)"
echo "  any 'ambiguous' lines:          $(grep -aciE 'ambiguous' /tmp/u666_c1.out)"
grep -aiE 'ambiguous|url_file_acquisition' /tmp/u666_c1.out | head -6 | sed 's/^/  C| /' | cut -c1-220
echo "=== C. gunbc compile of the two touched files directly:"
for f in $G1 $G2; do
  timeout 900 $GB compile --entry $f --source-root dag --source-root src/v2 > /tmp/u666_cf.out 2>&1
  echo "  [$f] rc=$? ambiguous=$(grep -aciE 'ambiguous' /tmp/u666_cf.out)"
  grep -aiE 'ambiguous|error' /tmp/u666_cf.out | head -3 | sed 's/^/  C| /' | cut -c1-220
done
git status --porcelain | head -3
echo "DONE_14666 $(date -u +%H:%M:%S)Z"
