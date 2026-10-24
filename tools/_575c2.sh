set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
# C2 ONLY: claim-coverage-census at 05a28f7d734 (dedicated; earlier attempts rc=124@1200s, then runner wall at ~33min in).
HEADSHA=05a28f7d7349707cf23e65c325322884dd0dc9e8
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575c2
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575n 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/f575n 2>/dev/null)" = "$HEADSHA" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/f575n 2>/dev/null) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575c2 $HEADSHA || { echo WT_FAIL; exit 9; }
WT=/tmp/wt575c2
sha(){ sha256sum "$1" | cut -d' ' -f1; }
cd $WT || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WT_NOT_AT_HEAD; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) START $(date -u +%H:%M:%S)Z"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t575c2 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t575c2/release/gunbc
[ -x $GB ] || { echo "NO gunbc -- nothing below is a result"; exit 9; }
echo "GB_SHA=$(sha $GB) built $(date -u +%H:%M:%S)Z"
( cd $WT && timeout 2400 $GB test //gunbc/instruments:claim-coverage-census ) > /tmp/c575_c2.out 2>&1; rc=$?
echo "ROW coverage-census rc=$rc END $(date -u +%H:%M:%S)Z"
if [ "$rc" = "124" ]; then
  echo "  UNOBSERVED: TIMEOUT at 2400s (attempt log: rc=124@1200s, wall@~33min). Size seen so far: $(wc -c < /tmp/c575_c2.out) bytes, $(wc -l < /tmp/c575_c2.out) lines"
  tail -4 /tmp/c575_c2.out | sed 's/^/  T| /' | cut -c1-190
else
  HELD=$(grep -ac ' held ' /tmp/c575_c2.out); FA=$(grep -acE '^FAIL ' /tmp/c575_c2.out); UN=$(grep -acE '^UNOBSERVED ' /tmp/c575_c2.out)
  echo "  held=$HELD FAIL=$FA UNOBS=$UN lines=$(wc -l < /tmp/c575_c2.out)"
  grep -aE '^FAIL |^UNOBSERVED ' /tmp/c575_c2.out | head -12 | sed 's/^/  V| /' | cut -c1-200
  grep -aiE 'summary|unreached|admitted|rows=' /tmp/c575_c2.out | tail -8 | sed 's/^/  S| /' | cut -c1-200
  [ "$rc" = "0" ] && [ "$((HELD+FA+UN))" = "0" ] && echo "  ROW coverage-census !! rc=0 zero-verdict VOID"
fi
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
echo DONE_575C2
