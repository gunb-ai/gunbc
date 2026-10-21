set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14666 instrument arm — HEAD only (the prior dispatch's 50m cap killed the runs; each ~29m).
# Verifies: gunbc compile route at head prints NO ambiguity error naming DeclarationRef or GcpPolicy.
HEADSHA=d3d44cfed33af0feede633829f7cf574f648ecc7
echo "HEADSHA=$HEADSHA START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w666h2
git fetch -q origin +refs/pull/14666/head:refs/verifier/14666h2 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/14666h2)" = "$HEADSHA" ] || { echo "PR_REF_MOVED STOP"; exit 9; }
git worktree add -q --detach /tmp/w666h2 $HEADSHA || { echo WT_FAIL; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cd /tmp/w666h2 || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WRONG_COMMIT; exit 9; }
( CARGO_TARGET_DIR=/tmp/w666h2/.t666 cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1 )
GB=/tmp/w666h2/.t666/release/gunbc; [ -x $GB ] || { echo NO_GUNBC; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) GUNBC_SHA=$(sha $GB)"
( CARGO_TARGET_DIR=/tmp/w666h2/.t666 timeout 2400 $GB test //gunbc/instruments:native-process-memory-census > /tmp/c666_gunbc_h.log 2>&1 ); rc=$?
echo "  gunbc-rc=$rc (author at head: rc 101, 1 PASS 8 SKIPPED; tail error #14607 separate)"
echo "  ambiguity_total=$(grep -ac 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log)"
echo "  DeclarationRef_hits(expect 0)=$(grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | grep -ac DeclarationRef)"
echo "  GcpPolicy_hits(expect 0)=$(grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | grep -ac GcpPolicy)"
grep -a 'ambiguous exact Rust binding' /tmp/c666_gunbc_h.log | head -5 | sed 's/^/    AMB| /' | cut -c1-190
echo "  PASS_count=$(grep -ac 'PASS ' /tmp/c666_gunbc_h.log) SKIP_count=$(grep -ac 'SKIPPED' /tmp/c666_gunbc_h.log)"
tail -8 /tmp/c666_gunbc_h.log | sed 's/^/    T| /' | cut -c1-170
echo DONE_666_HEAD
cd /root/workspace/repo-root
git worktree remove --force /tmp/w666h2 >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
