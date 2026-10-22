#!/usr/bin/env bash
# Batch re-verification at MOVED heads (post-receipt main merges, keep-both pattern):
# #14657 1a3815e78d1580f0d9c29c4e8d178b820fdf87cb (6 PR claims + fold 11)
# #14661 6dcfa5c93861932f4e233e7bf85441661bb55bea (1 PR claim + fold 11)
# #14668 85f8bc3c268b5d2bbe97eb7347fd9334a1020c66 (4 PR claims + fold 11)
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verbat4 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verbat4
echo 51539607552 > /sys/fs/cgroup/verbat4/memory.max
RB=$(cat /sys/fs/cgroup/verbat4/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verbat4 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verbat4/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tbat4
sha() { sha256sum "$1" | cut -d' ' -f1; }
[ "$(git rev-parse origin/main)" = "374326d848c32ddd588ce783526a214fae2550fd" ] || git fetch -q origin +refs/heads/main:refs/heads/tmp-main3 2>/dev/null || true

run_head() { # $1=PR $2=OID $3=wt $4=fetchref  then pairs of entry:namesfile
  local pr=$1 oid=$2 wt=$3
  rm -rf $wt; git worktree prune >/dev/null 2>&1 || true
  git fetch -q origin +refs/pull/$pr/head:refs/verifier/v$pr || { echo "[$pr] FETCH_FAIL"; return 1; }
  [ "$(git rev-parse refs/verifier/v$pr)" = "$oid" ] || { echo "[$pr] PR_REF_MOVED fetched=$(git rev-parse refs/verifier/v$pr) want=$oid -- SKIP"; return 1; }
  git worktree add -q --detach $wt $oid || { echo "[$pr] WT_FAIL"; return 1; }
  cd $wt || return 1
  [ "$(git rev-parse HEAD)" = "$oid" ] || { echo "[$pr] WT_NOT_AT_HEAD"; return 1; }
  for f in src/v2/workflow/local_repo_wet_terminal.dag src/v2/test/claim/local_repo_wet_terminal.dag; do
    [ "$(git cat-file blob $oid:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "[$pr] NOT_HEAD_BLOB $f -- SKIP"; return 1; }
  done
  cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  [ "$(git rev-parse HEAD)" = "$oid" ] || { echo "[$pr] BUILT_AT_WRONG_COMMIT"; return 1; }
  CB=$CARGO_TARGET_DIR/release/claim_batch
  [ -x $CB ] || { echo "[$pr] NO BINARY"; return 1; }
  echo "[$pr] BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) $(date -u +%H:%M:%S)Z"
  # fold-compile: all 11 claims of the fold claim file, names read from head file
  local names=$(grep -oE '^test fn [A-Za-z_0-9]+' src/v2/test/claim/local_repo_wet_terminal.dag | sed 's/^test fn //' | paste -sd,)
  echo "[$pr] fold names: $(echo $names | tr ',' ' ' | head -c 200)..."
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry src/v2/test/claim/local_repo_wet_terminal.dag --functions "$names" > /tmp/bat4_${pr}_fold.out 2>&1
  echo "[$pr] FOLD-11 rc=$? PASS=$(grep -ac '^PASS ' /tmp/bat4_${pr}_fold.out) FAIL=$(grep -ac '^FAIL ' /tmp/bat4_${pr}_fold.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/bat4_${pr}_fold.out)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/bat4_${pr}_fold.out | head -5 | sed 's/^/    V| /' | cut -c1-200
}
echo "=== 14657 @ 1a3815e78d: $(date -u +%H:%M:%S)Z"
run_head 14657 1a3815e78d1580f0d9c29c4e8d178b820fdf87cb /tmp/wbat4a || echo "14657 arm incomplete"
cd /tmp/wbat4a || exit 9
N657="documentary_refs_a_cited_row_the_document_omits_refuses,documentary_refs_a_ledger_only_row_the_document_cites_refuses,documentary_refs_agreeing_standings_are_admitted,documentary_refs_every_ruling_standing_agrees_with_the_document,documentary_refs_generated_document_renders_the_rows"
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry dag/test/claim/documentary_refs_witness.dag --functions "$N657" > /tmp/bat4_657_refs.out 2>&1
echo "[14657] refs-5 rc=$? PASS=$(grep -ac '^PASS ' /tmp/bat4_657_refs.out) FAIL=$(grep -ac '^FAIL ' /tmp/bat4_657_refs.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/bat4_657_refs.out)"
for f in ${N657//,/ }; do echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/bat4_657_refs.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/bat4_657_refs.out)"; done
timeout 900 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry dag/test/claim/design_ruling_standing_witness.dag --functions "an_unclassified_ruling_row_is_refused_and_a_classified_one_is_not" > /tmp/bat4_657_ruling.out 2>&1
echo "[14657] ruling rc=$? PASS=$(grep -ac '^PASS ' /tmp/bat4_657_ruling.out) FAIL=$(grep -ac '^FAIL ' /tmp/bat4_657_ruling.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/bat4_657_ruling.out)"
echo "=== 14661 @ 6dcfa5c938: $(date -u +%H:%M:%S)Z"
run_head 14661 6dcfa5c93861932f4e233e7bf85441661bb55bea /tmp/wbat4b || echo "14661 arm incomplete"
cd /tmp/wbat4b || exit 9
timeout 900 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry src/v2/test/claim/body_lowering/map_literal.dag --functions "ml_the_key_law_names_both_occurrences_holds" > /tmp/bat4_661.out 2>&1
echo "[14661] claim rc=$? PASS=$(grep -ac '^PASS ' /tmp/bat4_661.out) FAIL=$(grep -ac '^FAIL ' /tmp/bat4_661.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/bat4_661.out)"
grep -aE '^(FAIL|UNOBSERVED|PASS) ' /tmp/bat4_661.out | sed 's/^/    V| /' | cut -c1-200
echo "=== 14668 @ 85f8bc3c26: $(date -u +%H:%M:%S)Z"
run_head 14668 85f8bc3c268b5d2bbe97eb7347fd9334a1020c66 /tmp/wbat4c || echo "14668 arm incomplete"
cd /tmp/wbat4c || exit 9
N668="single_field_literal_passes_through_bare_value,single_field_ambiguous_literal_emits_the_refusal,multi_field_literal_emits_nominal_struct,multi_field_ambiguous_literal_refuses_with_located_diagnostic"
[ "$(git cat-file blob 85f8bc3c268b5d2bbe97eb7347fd9334a1020c66:dag/test/claim/record_emission_candidate_set_witness.dag | sha256sum | cut -d' ' -f1)" = "$(sha dag/test/claim/record_emission_candidate_set_witness.dag)" ] || echo "[14668] witness blob CHECK FAIL"
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry dag/test/claim/record_emission_candidate_set_witness.dag --functions "$N668" > /tmp/bat4_668.out 2>&1
echo "[14668] claims rc=$? PASS=$(grep -ac '^PASS ' /tmp/bat4_668.out) FAIL=$(grep -ac '^FAIL ' /tmp/bat4_668.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/bat4_668.out)"
for f in ${N668//,/ }; do echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/bat4_668.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/bat4_668.out)"; done
grep -a 'eval_steps' /tmp/bat4_668.out | head -4 | sed 's/^/    W| /' | cut -c1-200
cd $REPO
echo DONE_BAT4 $(date -u +%H:%M:%S)Z
