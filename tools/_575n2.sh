set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
# #14575 @05a28f7d734 D2: plant battery (P1/P2/P3b/R2 aimed) with the same head-built binary; R1b in n3.
HEADSHA=05a28f7d7349707cf23e65c325322884dd0dc9e8
CEN=src/v2/workflow/outside_discovery_roots_census.dag
WIT=src/v2/test/claim/outside_discovery_roots_census_witness.dag
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/wt575n2
git fetch -q origin +refs/pull/14575/head:refs/verifier/f575n 2>&1 | tail -1
[ "$(git rev-parse refs/verifier/f575n 2>/dev/null)" = "$HEADSHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
git worktree add -q --detach /tmp/wt575n2 $HEADSHA || { echo WT_FAIL; exit 9; }
WT=/tmp/wt575n2
sha(){ sha256sum "$1" | cut -d' ' -f1; }
cd $WT || exit 9
[ "$(git rev-parse HEAD)" = "$HEADSHA" ] || { echo WT_NOT_AT_HEAD; exit 9; }
[ "$(sha $WIT)" = "$(git cat-file blob $HEADSHA:$WIT | sha256sum | cut -d' ' -f1)" ] || { echo WIT_NOT_HEAD; exit 9; }
HEAD_CEN_SHA=$(sha $CEN)
echo "BUILD_COMMIT=$(git rev-parse HEAD) HEAD_CEN_SHA=$HEAD_CEN_SHA"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t575n2 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t575n2/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
cp -f $CEN /tmp/cen575n2_orig
STAGE=$REPO/tools/stage
[ -f $STAGE/census873_p1.dag ] || { echo "STAGE_PLANTS_MISSING -- STOP (n2 cannot run; plants lost)"; exit 9; }
# sanity: every plant differs from the head blob (a plant == head would be a silent non-mutation)
for p in census873_p1.dag census873_p2.dag census873_p3b.dag census873_r2.dag; do
  [ "$(sha $STAGE/$p)" != "$HEAD_CEN_SHA" ] && echo "  PLANT_OK $p" || echo "  !! PLANT_IS_HEAD $p -- STOP"; [ "$(sha $STAGE/$p)" = "$HEAD_CEN_SHA" ] && exit 9
done
AIM_P1="the_join_requires_the_fact_to_name_the_path_and_the_module_to_be_emitted"
AIM_P2="the_citation_list_covers_every_authority_the_cause_carries"
AIM_P3B="the_receipt_credits_the_note_only_at_the_grain_it_holds,every_receipt_line_says_the_marker_answer_is_a_pattern_match_not_the_marker_authority,the_census_receipt_never_borrows_the_marker_authoritys_name"
AIM_R="an_unreadable_listing_is_not_an_empty_census,a_listing_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_listing_answer_is_the_index_no_answer_arm,a_marker_read_the_process_refused_keeps_its_exit_code_and_stderr_as_data,an_empty_marker_answer_is_the_marker_no_answer_arm,the_six_unread_causes_render_to_six_distinct_receipt_texts,a_refused_exit_code_and_stderr_reach_the_rendered_receipt"
echo "=== aimed baseline (7 cause claims, no plant): $(date -u +%H:%M:%S)Z"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$AIM_R" > /tmp/n2_aimbase.out 2>&1
echo "  ROW aimbase rc=$? PASS=$(grep -ac '^PASS ' /tmp/n2_aimbase.out) FAIL=$(grep -ac '^FAIL ' /tmp/n2_aimbase.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/n2_aimbase.out)"
runaim() { # $1 plant $2 tag $3 names
  cp -f $STAGE/$1 $CEN
  echo "  [$2] census sha=$(sha $CEN | cut -c1-16) $(date -u +%H:%M:%S)Z"
  timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $WIT --functions "$3" > /tmp/n2_$2.out 2>&1
  local rc=$?; local NP NF NU; NP=$(grep -ac '^PASS ' /tmp/n2_$2.out); NF=$(grep -ac '^FAIL ' /tmp/n2_$2.out); NU=$(grep -ac '^UNOBSERVED ' /tmp/n2_$2.out)
  echo "  ROW $2 rc=$rc PASS=$NP FAIL=$NF UNOBS=$NU"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/n2_$2.out | sed 's/^/    F| /' | cut -c1-220
  cp -f /tmp/cen575n2_orig $CEN
  [ "$(sha $CEN)" = "$HEAD_CEN_SHA" ] && echo "    RESTORE_BYTE_IDENTICAL" || echo "    RESTORE_MISMATCH -- STOP"
}
echo "=== P1 join-conjunct:"; runaim census873_p1.dag p1 "$AIM_P1"
echo "=== P2 citation-list:"; runaim census873_p2.dag p2 "$AIM_P2"
echo "=== P3b receipt attribution:"; runaim census873_p3b.dag p3b "$AIM_P3B"
echo "=== R2 renderer drops exit_code/stderr:"; runaim census873_r2.dag r2 "$AIM_R"
git status --porcelain | head -3 | sed 's/^/  STALE| /'
echo DONE_575N2 $(date -u +%H:%M:%S)Z
