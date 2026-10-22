#!/usr/bin/env bash
# #14616 @ 6e78085bab9e — d2: Rust differential pin at head (cargo test, expect PASS), then
# MUTATION: EXECUTING_COMMIT_SURFACES ["GithubActionsCiJob"] -> ["GithubActionsCiJob","FalsifierCadenceJob"]
# (a mutated EXECUTING_SURFACES) => the pin must FAIL. Restore byte-exact, re-PASS.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=6e78085bab9edf9aed777f85568e00055672267c
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver616b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver616b
echo 51539607552 > /sys/fs/cgroup/ver616b/memory.max
RB=$(cat /sys/fs/cgroup/ver616b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver616b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver616b/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
CLR=src/v1/stage0/src/cli_run.rs
git fetch -q origin +refs/pull/14616/head:refs/verifier/616b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/616b)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/616b) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w616b 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w616b; git worktree add --detach /tmp/w616b $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w616b ] || { echo WT_FAIL; exit 9; }
cd /tmp/w616b || exit 9
[ "$(git cat-file blob $OID:$CLR | sha256sum | cut -d' ' -f1)" = "$(sha $CLR)" ] || { echo "NOT_HEAD_BLOB cli_run -- STOP"; exit 9; }
echo "HEADSHA=$OID"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t616b
echo "=== A. pin test at head (expect pass):"
timeout 3000 cargo test --release -p v1-compiler --lib executing_surfaces_mirror_enrollment_is_scheduled 2>&1 | tail -4 | sed 's/^/  T| /'
cp -f $CLR /tmp/clr616_head_orig
echo "=== B. MUTATION: EXECUTING_COMMIT_SURFACES gains FalsifierCadenceJob (rebuild via test):"
python3 - <<'PY'
p='src/v1/stage0/src/cli_run.rs'
s=open(p).read()
old='const EXECUTING_COMMIT_SURFACES: [&str; 1] = ["GithubActionsCiJob"];'
new='const EXECUTING_COMMIT_SURFACES: [&str; 2] = ["GithubActionsCiJob", "FalsifierCadenceJob"];'
n=s.count(old)
print("  mutation anchor count:",n)
assert n==1, n
open(p,'w').write(s.replace(old,new))
PY
echo "  mutated sha=$(sha $CLR) orig=$(sha /tmp/clr616_head_orig)"
timeout 3000 cargo test --release -p v1-compiler --lib executing_surfaces_mirror_enrollment_is_scheduled 2>&1 | tail -8 | sed 's/^/  T| /'
echo "=== C. restore byte-exact + re-run pin (expect pass):"
cp -f /tmp/clr616_head_orig $CLR
[ "$(sha $CLR)" = "$(sha /tmp/clr616_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"
timeout 3000 cargo test --release -p v1-compiler --lib executing_surfaces_mirror_enrollment_is_scheduled 2>&1 | tail -4 | sed 's/^/  T| /'
git status --porcelain | head -3
echo "DONE_14616_D2 $(date -u +%H:%M:%S)Z"
