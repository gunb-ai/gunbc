set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14613 @b7854d98d76 census re-arm with corrected extraction (manager check: census ADMITS the edited row).
OID=b7854d98d76584c65b04df7ce1a971b1ab7322fc
echo "HEADSHA_from_gh=$OID START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w613j
git fetch -q origin +refs/pull/14613/head:refs/verifier/14613g 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/14613g 2>/dev/null)" = "$OID" ] || { echo "PR_REF_MOVED=$(git rev-parse refs/verifier/14613g 2>/dev/null) -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w613j $OID || { echo WT_FAILED; exit 9; }
WT=/tmp/w613j
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo WT_MISMATCH; exit 9; }
cd $WT || exit 9
sha() { sha256sum "$1" | cut -d' ' -f1; }
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14613j CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
GB=/root/workspace/repo-root/.t14613j/release/gunbc
[ -x $GB ] || { echo "NO gunbc -- nothing below is a result"; exit 9; }
echo "GB_SHA=$(sha $GB)"
echo "-------- census full capture $(date -u +%H:%M:%S)Z"
( cd $WT && timeout 2400 $GB test //gunbc/instruments:rfm-rung-census ) > /tmp/c613j_census.out 2>&1; rc=$?
LINES=$(wc -l < /tmp/c613j_census.out); BYTES=$(wc -c < /tmp/c613j_census.out)
HELD=$(grep -ac ' held' /tmp/c613j_census.out); REFUSED=$(grep -ac 'refused' /tmp/c613j_census.out)
NP=$(grep -ac '^PASS ' /tmp/c613j_census.out); NF=$(grep -ac '^FAIL ' /tmp/c613j_census.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/c613j_census.out)
echo "  ROW census rc=$rc lines=$LINES bytes=$BYTES PASS=$NP FAIL=$NF UNOBS=$NU held=$HELD refused=$REFUSED"
grep -aE '^(PASS|FAIL|UNOBSERVED) |held|refused' /tmp/c613j_census.out | head -25 | sed 's/^/  V| /' | cut -c1-190
grep -aiE 'rows=|read=|roster |cases|summary' /tmp/c613j_census.out | tail -8 | sed 's/^/  S| /' | cut -c1-195
echo "  -- lines naming the EDITED row (identity or file):"
grep -a 'workflow_expressions_assembled_from_string_fragments' /tmp/c613j_census.out | head -6 | sed 's/^/  R| /' | cut -c1-230
echo "  edited-name census line count=$(grep -ac 'workflow_expressions_assembled_from_string_fragments' /tmp/c613j_census.out)"
[ "$rc" = "0" ] && [ "$((NP+NF+NU+HELD))" = "0" ] && echo "  ROW census !! rc=0 with ZERO case lines -- VOID"
tail -6 /tmp/c613j_census.out | sed 's/^/  T| /' | cut -c1-195
echo DONE_613_J
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
