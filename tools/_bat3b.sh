set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Batch at NEW heads (manager msg 16:5xZ): #14657 b54d4894264 + #14661 71dfd2999e7.
# Per head: fetch + PR_REF_MOVED guard vs gh-resolved OID; worktree; head-built CB (commit+sha);
# fold-compile arm (src/v2/test/claim/local_repo_wet_terminal.dag, all names, wet, ENROLLED route
# dag+src/v2 -- parses+audits every schedule row; dup-field/empty-row would land all-UNOBSERVED
# parse-failed); then the PR's claims BY NAME on the ENROLLED route (2 roots, per fleet doc :187).
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver3a 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver3a
echo 51539607552 > /sys/fs/cgroup/ver3a/memory.max
RB=$(cat /sys/fs/cgroup/ver3a/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver3a memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver3a/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
LCW=src/v2/test/claim/local_repo_wet_terminal.dag
runpr() { # pr oid tag entry1 fns1 [entry2 fns2]
  local pr=$1 oid=$2 tag=$3
  echo "======== #$pr @ ${oid:0:11} [$tag] $(date -u +%H:%M:%S)Z"
  rm -rf /tmp/wb3
  git fetch -q origin +refs/pull/$pr/head:refs/verifier/b3$pr || { echo "  FETCH_FAIL"; return 1; }
  [ "$(git rev-parse refs/verifier/b3$pr)" = "$oid" ] || { echo "  PR_REF_MOVED fetched=$(git rev-parse refs/verifier/b3$pr) want=$oid"; return 1; }
  git worktree add -q --detach /tmp/wb3 $oid || { echo "  WT_FAIL"; return 1; }
  cd /tmp/wb3 || return 1
  [ "$(git cat-file blob $oid:$LCW | sha256sum | cut -d' ' -f1)" = "$(sha $LCW)" ] || { echo "  LCW_NOT_HEAD_BLOB"; return 1; }
  CARGO_TARGET_DIR=/root/workspace/repo-root/.tb3 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  [ "$(git rev-parse HEAD)" = "$oid" ] || { echo "  BUILT_AT_WRONG_COMMIT"; return 1; }
  local CB=/root/workspace/repo-root/.tb3/release/claim_batch
  [ -x $CB ] || { echo "  NO BINARY -- nothing below is a result"; return 1; }
  echo "  BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
  # fold-compile arm
  local FOLDN=$(grep -aoE '^test fn [a-zA-Z0-9_]+' $LCW | sed 's/^test fn //' | sort | paste -sd,)
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $LCW --functions "$FOLDN" > /tmp/cb3_${tag}_fold.out 2>&1
  local rc=$?
  echo "  FOLD rc=$rc PASS=$(grep -ac '^PASS ' /tmp/cb3_${tag}_fold.out) FAIL=$(grep -ac '^FAIL ' /tmp/cb3_${tag}_fold.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/cb3_${tag}_fold.out) (11 expected)"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/cb3_${tag}_fold.out | head -4 | sed 's/^/    V| /' | cut -c1-200
  # PR claims, entry 1
  local entry=$4 fns=$5
  [ "$(git cat-file blob $oid:$entry | sha256sum | cut -d' ' -f1)" = "$(sha $entry)" ] || { echo "  ENTRY_NOT_HEAD_BLOB $entry"; return 1; }
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $entry --functions "$fns" > /tmp/cb3_${tag}_c1.out 2>&1
  rc=$?
  echo "  [c1 $entry] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/cb3_${tag}_c1.out) FAIL=$(grep -ac '^FAIL ' /tmp/cb3_${tag}_c1.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/cb3_${tag}_c1.out)"
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/cb3_${tag}_c1.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
  grep -aE '^(FAIL|UNOBSERVED) ' /tmp/cb3_${tag}_c1.out | head -6 | sed 's/^/    V| /' | cut -c1-200
  echo "$fns" | tr ',' '\n' | while read -r f; do
    echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/cb3_${tag}_c1.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/cb3_${tag}_c1.out)"
  done
  # optional entry 2
  if [ $# -ge 7 ]; then
    local entry2=$6 fns2=$7
    [ "$(git cat-file blob $oid:$entry2 | sha256sum | cut -d' ' -f1)" = "$(sha $entry2)" ] || { echo "  ENTRY_NOT_HEAD_BLOB $entry2"; return 1; }
    timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $entry2 --functions "$fns2" > /tmp/cb3_${tag}_c2.out 2>&1
    rc=$?
    echo "  [c2 $entry2] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/cb3_${tag}_c2.out) FAIL=$(grep -ac '^FAIL ' /tmp/cb3_${tag}_c2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/cb3_${tag}_c2.out)"
    grep -aE '^(FAIL|UNOBSERVED) ' /tmp/cb3_${tag}_c2.out | head -4 | sed 's/^/    V| /' | cut -c1-200
    echo "$fns2" | tr ',' '\n' | while read -r f; do
      echo "    $f: PASS=$(grep -ac "^PASS $f\$" /tmp/cb3_${tag}_c2.out) FAIL=$(grep -ac "^FAIL $f\$" /tmp/cb3_${tag}_c2.out)"
    done
  fi
  cd /root/workspace/repo-root
  git worktree remove --force /tmp/wb3 >/dev/null 2>&1 || true
  git worktree prune >/dev/null 2>&1 || true
  return 0
}
runpr 14668 7c97832508ad532bcc996b5ac520b862c43cb33f 14668n \
  dag/test/claim/record_emission_candidate_set_witness.dag "single_field_ambiguous_literal_emits_the_refusal,multi_field_ambiguous_literal_refuses_with_located_diagnostic,single_field_literal_passes_through_bare_value,multi_field_literal_emits_nominal_struct"
runpr 14671 fe0cb8328290137f2051890cdec36ccb985c12d0 14671n \
  dag/test/claim/ledger_row_coherence.dag "a_coherent_artifact_declares_no_repair,a_coherent_artifact_still_declares_that_it_was_judged,a_declaration_matching_the_inference_agrees,a_declaration_that_over_claims_is_caught_in_the_declared_direction,a_difference_that_drops_no_row_is_not_reported_incoherent,a_fully_rendered_ledger_is_coherent,a_non_ledger_artifact_cannot_report_coherent,a_rendered_declaration_parses_back_to_the_same_identities,a_repair_that_under_declares_is_caught_in_the_inferred_direction,a_repairing_declaration_reports_both_numerator_and_denominator,a_trailer_missing_its_identity_yields_no_declaration,a_withheld_row_is_named_as_missing,an_absent_declaration_judges_nothing,an_unjudgeable_ledger_refuses_rather_than_skipping,an_unjudgeable_non_ledger_artifact_does_not_stop_the_line,one_judged_ledger_does_not_license_a_commit_over_an_unjudged_one,the_gate_diagnostic_names_the_missing_identity,the_rung_drop_ledger_is_covered_without_a_projection_index"
echo DONE_B3B
