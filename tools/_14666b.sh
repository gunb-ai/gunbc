#!/usr/bin/env bash
# #14666 @ 5a4840b9d70d — d2 (fixup): correct gunbc compile invocation (usage error last time).
# Print help, then compile the ledger entry and the two touched files; scan for ambiguity errors.
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
rm -rf /sys/fs/cgroup/ver666b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver666b
echo 51539607552 > /sys/fs/cgroup/ver666b/memory.max
RB=$(cat /sys/fs/cgroup/ver666b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver666b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver666b/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
LC=dag/test/claim/ledger_row_coherence.dag
G1=dag/extdeps/realization/reconcile_in_process.dag
G2=dag/extdeps/cloud/gcp/storage.dag
git fetch -q origin +refs/pull/14666/head:refs/verifier/666b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/666b)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/666b) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w666b 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w666b; git worktree add --detach /tmp/w666b $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w666b ] || { echo WT_FAIL; exit 9; }
cd /tmp/w666b || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t666
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
GB=/root/workspace/repo-root/.t666/release/gunbc
[ -x $GB ] || { echo "NO BINARY"; exit 9; }
echo "HEADSHA=$(git rev-parse HEAD) GB_SHA=$(sha $GB) built $(date -u +%H:%M:%S)Z"
echo "=== gunbc compile --help:"
timeout 120 $GB compile --help 2>&1 | head -20 | sed 's/^/  H| /'
echo "=== compile ledger entry (whole-corpus shape: first source-root is the root):"
timeout 2400 $GB compile --source-root dag --source-root src/v2 --entry $LC --target dag > /tmp/v666_c1.out 2>&1
echo "  rc=$? bytes=$(wc -c < /tmp/v666_c1.out)"
echo "  DeclarationRef-ambiguity=$(grep -aciE 'ambiguous.*RustItemDeclarationRef' /tmp/v666_c1.out) GcpPolicy-ambiguity=$(grep -aciE 'ambiguous.*GcsBucketPolicyWire' /tmp/v666_c1.out) any-ambiguous=$(grep -aciE 'ambiguous' /tmp/v666_c1.out)"
grep -aiE 'ambiguous|url_file_acquisition' /tmp/v666_c1.out | head -6 | sed 's/^/  C| /' | cut -c1-220
echo "=== compile the two touched files as entries:"
for f in $G1 $G2; do
  timeout 900 $GB compile --source-root dag --source-root src/v2 --entry $f --target dag > /tmp/v666_cf.out 2>&1
  echo "  [$f] rc=$? bytes=$(wc -c < /tmp/v666_cf.out) ambiguous=$(grep -aciE 'ambiguous' /tmp/v666_cf.out)"
  grep -aiE 'ambiguous' /tmp/v666_cf.out | head -3 | sed 's/^/  C| /' | cut -c1-220
done
git status --porcelain | head -3
echo "DONE_14666_D2 $(date -u +%H:%M:%S)Z"
