#!/usr/bin/env bash
# #14666 @ 5a4840b9d70d — d3 (fixup 2): gunbc compile with the REQUIRED --output-dir, plus a
# BASE arm so the ambiguity scan is proven non-vacuous: at base the two ambiguity errors must
# APPEAR; at head they must be gone. Record the remaining url_file_acquisition tail error (#14607).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=5a4840b9d70d3c33904236ec27eb12f6097dcc69
BASEP=201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1  # gh baseRefOid == merge-base(v14666, main)
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver666c 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver666c
echo 51539607552 > /sys/fs/cgroup/ver666c/memory.max
RB=$(cat /sys/fs/cgroup/ver666c/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver666c memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver666c/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
LC=dag/test/claim/ledger_row_coherence.dag
git fetch -q origin +refs/pull/14666/head:refs/verifier/666c || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/666c)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/666c) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w666c 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w666c; git worktree add --detach /tmp/w666c $OID 2>&1 | tail -1 | sed 's/^/  WT| /'
cd /tmp/w666c || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t666
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
GB=/root/workspace/repo-root/.t666/release/gunbc
[ -x $GB ] || { echo "NO BINARY"; exit 9; }
echo "HEADSHA=$(git rev-parse HEAD) GB_SHA=$(sha $GB) built $(date -u +%H:%M:%S)Z"
comp() { # wt tag
  local wt=$1 tag=$2
  cd $wt || { echo "  [$tag] DIR_FAIL"; return; }
  rm -rf /tmp/out666_$tag; mkdir -p /tmp/out666_$tag
  timeout 2400 $GB compile --source-root dag --source-root src/v2 --entry $LC --target dag --output-dir /tmp/out666_$tag > /tmp/w666_${tag}.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc bytes=$(wc -c < /tmp/w666_${tag}.out) emitted=$(find /tmp/out666_$tag -type f 2>/dev/null | wc -l)"
  echo "  DeclarationRef-vs-RustItemDeclarationRef=$(grep -acE 'RustItemDeclarationRef' /tmp/w666_${tag}.out) GcpPolicy-vs-GcsBucketPolicyWire=$(grep -acE 'GcsBucketPolicyWire' /tmp/w666_${tag}.out) any_ambiguous=$(grep -aciE 'ambiguous' /tmp/w666_${tag}.out) url_file_acquisition=$(grep -ac 'url_file_acquisition' /tmp/w666_${tag}.out)"
  grep -aiE 'ambiguous|RustItemDeclarationRef|GcsBucketPolicyWire|url_file_acquisition' /tmp/w666_${tag}.out | head -8 | sed 's/^/  E| /' | cut -c1-230
}
echo "=== A. HEAD: compile the ledger entry (expect the two ambiguity errors ABSENT):"
comp /tmp/w666c head
echo "=== B. BASE $BASEP (same head-built binary; expect the two ambiguity errors PRESENT):"
git worktree remove --force /tmp/b666c 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/b666c; git worktree add --detach /tmp/b666c $BASEP 2>&1 | tail -1 | sed 's/^/  WT| /'
if [ -d /tmp/b666c ]; then
  comp /tmp/b666c base
else
  echo "  BASE_WT_FAIL -- scan stays vacuous; report honestly"
fi
echo "=== C. usage sanity: same command WITHOUT --output-dir must be a usage error (proves d2's 235B was that):"
cd /tmp/w666c || exit 9
timeout 300 $GB compile --source-root dag --source-root src/v2 --entry $LC --target dag > /tmp/w666_nousage.out 2>&1
echo "  rc=$? bytes=$(wc -c < /tmp/w666_nousage.out)"; head -2 /tmp/w666_nousage.out | sed 's/^/  U| /' | cut -c1-160
git status --porcelain | head -3
echo "DONE_14666_D3 $(date -u +%H:%M:%S)Z"
