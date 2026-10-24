set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
# #14575 @05a28f7d734 D3: R1b true-collision plant (aimed 7) + static greps + census spot-checks (main moved).
HEADSHA=05a28f7d7349707cf23e65c325322884dd0dc9e8
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575n3
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575n 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/f575n 2>/dev/null)" = "$HEADSHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575n3 $HEADSHA || { echo WT_FAIL; exit 9; }
WT=/tmp/wt575n3
sha(){ sha256sum "$1" | cut -d' ' -f1; }
cd $WT || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WT_NOT_AT_HEAD; exit 9; }
HEAD_CEN_SHA=$(sha $CEN)
echo "BUILD_COMMIT=$(git rev-parse HEAD)"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t575n3 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
CB=/root/workspace/repo-root/.t575n3/release/claim_batch
GB=/root/workspace/repo-root/.t575n3/release/gunbc
[ -x $CB ] && [ -x $GB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB) GB_SHA=$(sha $GB) built $(date -u +%H:%M:%S)Z"
cp -f $CEN /tmp/cen575n3_orig
STAGE=$REPO/tools/stage
[ -f $STAGE/census873_r1b.dag ] || { echo "R1B_PLANT_MISSING -- STOP"; exit 9; }
[ "$(sha $STAGE/census873_r1b.dag)" != "$HEAD_CEN_SHA" ] || { echo "PLANT_IS_HEAD -- STOP"; exit 9; }
AIM_R="an_unreadable_listing_is_not_an_empty_census,a_listing_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_listing_answer_is_the_index_no_answer_arm,a_marker_read_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_marker_answer_is_the_marker_no_answer_arm,the_six_unread_causes_render_to_six_distinct_receipt_texts,a_refused_exit_code_and_stderr_reach_the_rendered_receipt"
echo "=== S1 static greps (renderer boundary-only; no String-typed cause):"
echo "  S1a String cause hits: $(git grep -c 'cause: String' -- $CEN | wc -l) (expect 0)"
echo "  S1b renderer hits:"; git grep -n 'unread_cause_label\|CensusUnreadCause' -- $CEN | sed 's/^/    G| /' | cut -c1-160
git grep -n 'CensusUnreadCause::' -- 'src/v2/**' | grep -av outside_discovery_roots_census | head -6 | sed 's/^/    G| /' | cut -c1-160
echo "=== R1b true collision (plant replaces census subject):"
cp -f $STAGE/census873_r1b.dag $CEN
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$AIM_R" > /tmp/n3_r1b.out 2>&1; rc=$?
NP=$(grep -ac '^PASS ' /tmp/n3_r1b.out); NF=$(grep -ac '^FAIL ' /tmp/n3_r1b.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/n3_r1b.out)
echo "  ROW r1b rc=$rc PASS=$NP FAIL=$NF UNOBS=$NU"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/n3_r1b.out | sed 's/^/    F| /' | cut -c1-220
cp -f /tmp/cen575n3_orig $CEN
[ "$(sha $CEN)" = "$HEAD_CEN_SHA" ] && echo "    RESTORE_BYTE_IDENTICAL" || echo "    RESTORE_MISMATCH -- STOP"
echo "=== C1 rfm-rung-census at merge head $(date -u +%H:%M:%S)Z"
( cd $WT && timeout 1200 $GB test //gunbc/instruments:rfm-rung-census ) > /tmp/n3_census1.out 2>&1; rc=$?
HELD=$(grep -ac ' held ' /tmp/n3_census1.out); FA=$(grep -acE '^FAIL ' /tmp/n3_census1.out); UN=$(grep -acE '^UNOBSERVED ' /tmp/n3_census1.out)
echo "  ROW rfm-census rc=$rc held=$HELD FAIL=$FA UNOBS=$UN"
grep -aE '^FAIL |^UNOBSERVED |live_roster' /tmp/n3_census1.out | head -10 | sed 's/^/  V| /' | cut -c1-200
[ "$rc" = "0" ] && [ "$((HELD+FA+UN))" = "0" ] && echo "  ROW rfm-census !! rc=0 zero-verdict VOID"
echo "=== C2 claim-coverage-census at merge head $(date -u +%H:%M:%S)Z"
( cd $WT && timeout 1200 $GB test //gunbc/instruments:claim-coverage-census ) > /tmp/n3_census2.out 2>&1; rc=$?
HELD=$(grep -ac ' held ' /tmp/n3_census2.out); FA=$(grep -acE '^FAIL ' /tmp/n3_census2.out); UN=$(grep -acE '^UNOBSERVED ' /tmp/n3_census2.out)
echo "  ROW coverage-census rc=$rc held=$HELD FAIL=$FA UNOBS=$UN"
grep -aE '^FAIL |^UNOBSERVED ' /tmp/n3_census2.out | head -12 | sed 's/^/  V| /' | cut -c1-200
grep -aiE 'summary|unreached|admitted|rows=' /tmp/n3_census2.out | tail -6 | sed 's/^/  S| /' | cut -c1-200
[ "$rc" = "0" ] && [ "$((HELD+FA+UN))" = "0" ] && echo "  ROW coverage-census !! rc=0 zero-verdict VOID"
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_575N3 $(date -u +%H:%M:%S)Z
