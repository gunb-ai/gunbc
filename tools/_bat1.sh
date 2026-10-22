set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Identity-set batch: #14657 839097e (6 wet rows), #14661 8cfd351 (1), #14668 882db05 (2),
# #14671 cd7af89 (18). Per PR: dedicated ref fetch + PR_REF_MOVED guard vs the gh-resolved OID
# below; worktree at the OID; cargo build --release claim_batch AT that head (BUILD_COMMIT+
# CB_SHA printed); run the added wet rows' claims BY NAME (names read from each entry file at
# run time), wet route --source-root dag src/v2 src/v1. All runs inside this dispatch's bounded
# cgroup subtree (memory.max read-back asserted != max).
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verbat 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verbat
echo 51539607552 > /sys/fs/cgroup/verbat/memory.max
RB=$(cat /sys/fs/cgroup/verbat/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verbat memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verbat/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
runpr() { # pr oid entryfile fnlist tag
  local pr=$1 oid=$2 entry=$3 fns=$4 tag=$5
  echo "======== #$pr @ ${oid:0:11} [$tag] $(date -u +%H:%M:%S)Z"
  rm -rf /tmp/wbat
  git fetch -q origin +refs/pull/$pr/head:refs/verifier/bat$pr || { echo "  FETCH_FAIL"; return 1; }
  [ "$(git rev-parse refs/verifier/bat$pr)" = "$oid" ] || { echo "  PR_REF_MOVED fetched=$(git rev-parse refs/verifier/bat$pr) want=$oid"; return 1; }
  git worktree add -q --detach /tmp/wbat $oid || { echo "  WT_FAIL"; return 1; }
  cd /tmp/wbat || return 1
  [ "$(git cat-file blob $oid:$entry | sha256sum | cut -d' ' -f1)" = "$(sha $entry)" ] || { echo "  ENTRY_NOT_HEAD_BLOB"; return 1; }
  CARGO_TARGET_DIR=/root/workspace/repo-root/.tbat cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  [ "$(git rev-parse HEAD)" = "$oid" ] || { echo "  BUILT_AT_WRONG_COMMIT"; return 1; }
  local CB=/root/workspace/repo-root/.tbat/release/claim_batch
  [ -x $CB ] || { echo "  NO BINARY -- nothing below is a result"; return 1; }
  echo "  BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
  # names from the FILE at run time intersected with the expected list
  local have=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $entry | sed 's/^test fn //' | sort | paste -sd,)
  local want=$(echo "$fns" | tr ',' '\n' | sort | paste -sd,)
  [ "$have" = "$want" ] || echo "  (note: file-name-set differs from expected; running the expected names)"
  timeout 1700 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $entry --functions "$fns" > /tmp/cbat_$tag.out 2>&1
  local rc=$?
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/cbat_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/cbat_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/cbat_$tag.out)"
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/cbat_$tag.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/cbat_$tag.out | head -8 | sed 's/^/    V| /' | cut -c1-180
  echo "  verdict-by-name counts:"
  echo "$fns" | tr ',' '\n' | while read -r f; do
    local n=$(grep -ac "^PASS $f\$" /tmp/cbat_$tag.out); local x=$(grep -ac "^FAIL $f\$" /tmp/cbat_$tag.out)
    echo "    $f: PASS=$n FAIL=$x"
  done
  cd /root/workspace/repo-root
  git worktree remove --force /tmp/wbat >/dev/null 2>&1 || true
  git worktree prune >/dev/null 2>&1 || true
  return 0
}
runpr 14657 839097ec20a0ebba7788c5d9a1f81d744e54df31 dag/test/claim/documentary_refs_witness.dag \
  "documentary_refs_a_cited_row_the_document_omits_refuses,documentary_refs_a_ledger_only_row_the_document_cites_refuses,documentary_refs_agreeing_standings_are_admitted,documentary_refs_every_ruling_standing_agrees_with_the_document,documentary_refs_generated_document_renders_the_rows" 14657a
runpr 14657 839097ec20a0ebba7788c5d9a1f81d744e54df31 dag/test/claim/design_ruling_standing_witness.dag \
  "an_unclassified_ruling_row_is_refused_and_a_classified_one_is_not" 14657b
runpr 14661 8cfd35157ba2625dd609685afc89361044432917 src/v2/test/claim/body_lowering/map_literal.dag \
  "ml_the_key_law_names_both_occurrences_holds" 14661a
echo DONE_BATCH_1
