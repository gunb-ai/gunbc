# Common prelude for #14956 dispatches. Source it. Assumes: bash, run under `bash -lc` on the runner.
# Ensures the toolkit repo (this file's parent dir) is fresh, refs present, worktrees exist, binary built.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=1a2cfbedf763df63e5713d0f84d1a7b0e14dfd82
BASE_SHA=115143a4d8a1e870124e7848fa5084c3fcfe1c38
sha() { sha256sum "$1" | cut -d' ' -f1; }
LOGD=$REPO/.c4logs; mkdir -p $LOGD
# fetch toolkit updates if this run was launched from an older clone
git fetch -q origin session/verif-disp2 >/dev/null 2>&1 || true
# refs: PR head + base commit reachable via the PR head (base is its ancestor)
git fetch -q origin +refs/pull/14956/head:refs/verifier/f14956b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14956b)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14956b) want=$HEAD_SHA -- STOP"; exit 9; }
if [ -f $REPO/.git/shallow ]; then echo "shallow repo: unshallow gunbc remote"; git fetch -q --unshallow origin 2>/dev/null || git fetch -q origin main 2>/dev/null || true; fi
git cat-file -e $BASE_SHA^{commit} 2>/dev/null || git fetch -q origin main || true
git cat-file -e $BASE_SHA^{commit} >/dev/null 2>&1 || { echo "BASE_SHA_UNAVAILABLE -- STOP"; exit 9; }
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t956
H=/tmp/w956h; P=/tmp/w956p; M=/tmp/w956m; V=/tmp/w956v
wt () { # wt <path> <sha>  (idempotent; --force keeps content truthful at a pinned sha)
  [ -d $1/.git ] || [ -f $1/.git ] || git worktree add -q --force --detach $1 $2 || return 9
}
git worktree prune >/dev/null 2>&1 || true
build_head () {
  (cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_FAIL; return 9; }
  [ -x $CARGO_TARGET_DIR/release/claim_batch ] || { echo BUILD_MISSING; return 9; }
  echo "BUILD_COMMIT=$HEAD_SHA"; echo "HEAD_BIN sha256=$(sha $CARGO_TARGET_DIR/release/claim_batch)"
}
arm_cgroup () { # arm_cgroup <tag> -> sets CGPATH, bound via caller subshell
  local cg=/sys/fs/cgroup/ver956-$1
  rm -rf $cg 2>/dev/null || true; mkdir -p $cg || return 9
  echo 51539607552 > $cg/memory.max || return 9
  local RB; RB=$(cat $cg/memory.max)
  { [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEM_READBACK_FAIL $1 got=$RB"; return 9; }
  CGPATH=$cg
}
fns_of2 () { perl -0777 -ne 'while(/test fn\s+([A-Za-z0-9_]+)\s*\(/g){print "$1\n"}' $1 | sort -u; }
