#!/usr/bin/env bash
# #14792 @ d4683097f91da1f2e84abb0085e25b97c86d0e4e (base ed5930a0cc834b1c7d89b1c300cff8d680467687)
# 5 claims (3 rung_climb_witness + 2 rfm_rung_census_unread_cause_witness): head 5/5 with the
# SAME head-built binary; base must refuse (compile error, name not found). Statics: no other
# code consumer of the 4 renamed names.
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEAD=d4683097f91da1f2e84abb0085e25b97c86d0e4e
BASE=ed5930a0cc834b1c7d89b1c300cff8d680467687
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver792 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver792
echo 51539607552 > /sys/fs/cgroup/ver792/memory.max
RB=$(cat /sys/fs/cgroup/ver792/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver792 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver792/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t792
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w792h /tmp/w792b
git fetch -q origin +refs/pull/14792/head:refs/verifier/f792 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f792)" = "$HEAD" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f792) want=$HEAD -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w792h $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w792h || exit 9
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo WT_NOT_AT_HEAD; exit 9; }
RU=dag/test/claim/rung_climb_witness.dag
RF=dag/test/claim/rfm_rung_census_unread_cause_witness.dag
for f in $RU $RF; do [ "$(git cat-file blob $HEAD:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t792/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NRU="a_cannot_climb_further_standing_below_its_ceiling_is_refused_at_compile,a_can_climb_standing_below_its_ceiling_adds_no_refusal,a_prose_grounding_beside_the_trigger_capability_is_unwritable"
NRF="a_cause_arm_missing_from_the_labelling_match_is_refused_at_compile,the_same_match_with_a_catch_all_draws_no_non_exhaustive_refusal"
echo "=== HEAD 5 claims by name: $(date -u +%H:%M:%S)Z"
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $RU --functions "$NRU" > /tmp/f792_head_ru.out 2>&1; echo "[head RU] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f792_head_ru.out) FAIL=$(grep -ac '^FAIL ' /tmp/f792_head_ru.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f792_head_ru.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f792_head_ru.out | sed 's/^/  V| /' | cut -c1-200
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $RF --functions "$NRF" > /tmp/f792_head_rf.out 2>&1; echo "[head RF] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f792_head_rf.out) FAIL=$(grep -ac '^FAIL ' /tmp/f792_head_rf.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f792_head_rf.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f792_head_rf.out | sed 's/^/  V| /' | cut -c1-200
echo "=== BASE same binary (expect refusal; names gone at base? NO -- base still imports retired names -> compile refuse): $(date -u +%H:%M:%S)Z"
git worktree add -q --detach /tmp/w792b $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w792b || exit 9
[ "$(git rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
for f in $RU $RF; do [ "$(git cat-file blob $BASE:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || echo "  BASE_BLOB_MISMATCH $f"; done
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $RU --functions "$NRU" > /tmp/f792_base_ru.out 2>&1; echo "[base RU] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f792_base_ru.out) FAIL=$(grep -ac '^FAIL ' /tmp/f792_base_ru.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f792_base_ru.out) bytes=$(wc -c < /tmp/f792_base_ru.out)"
head -6 /tmp/f792_base_ru.out | sed 's/^/  B| /' | cut -c1-220
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --entry $RF --functions "$NRF" > /tmp/f792_base_rf.out 2>&1; echo "[base RF] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f792_base_rf.out) FAIL=$(grep -ac '^FAIL ' /tmp/f792_base_rf.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f792_base_rf.out) bytes=$(wc -c < /tmp/f792_base_rf.out)"
head -6 /tmp/f792_base_rf.out | sed 's/^/  B| /' | cut -c1-220
echo "=== statics: consumers of the 4 renamed names at HEAD (code lines only):"
for n in blocking_subject_count_for total_blocking_count_for blocking_class_count_for any_class_count_for; do
  raw=$(git grep -l "$n" $HEAD -- '*.dag' '*.rs' 2>/dev/null | wc -l)
  # code consumers: lines that are NOT inside string literals/comments -- show file:line and inspect
  hits=$(git grep -n "$n" $HEAD -- '*.dag' '*.rs' 2>/dev/null | grep -avE '":|"[^"]*'"$n"'[^"]*"|//|^\S+:[0-9]+:\s*(\*|///?)' | wc -l)
  echo "  $n files=$raw code_line_hits=$hits"
  git grep -n "$n" $HEAD -- '*.dag' '*.rs' 2>/dev/null | grep -avE '":|"[^"]*'"$n"'[^"]*"|//|^\S+:[0-9]+:\s*(\*|///?)' | head -3 | sed 's/^/    C| /' | cut -c1-200
done
echo "  head-imports-of-retired-names-in-claimed-files: $(grep -ac 'blocking_class_count_for\|total_blocking_count_for' $RU $RF | paste -sd' ' -)"
git status --porcelain | head -3
echo DONE_792 $(date -u +%H:%M:%S)Z
