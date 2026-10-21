set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Route correction re-run: #14657b and #14668a claims on the ENROLLED fleet route
# (--wet --source-root dag --source-root src/v2, per local_repo_wet_terminal.dag:187).
# My first runs added --source-root src/v1 (deviation). Full cause lines captured (600 cols).
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verrt 2>/dev/null || true; mkdir -p /sys/fs/cgroup/verrt
echo 51539607552 > /sys/fs/cgroup/verrt/memory.max
RB=$(cat /sys/fs/cgroup/verrt/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verrt memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verrt/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
runc() { # pr oid entry fns tag
  local pr=$1 oid=$2 entry=$3 fns=$4 tag=$5
  echo "======== #$pr ${oid:0:11} [$tag] enrolled route $(date -u +%H:%M:%S)Z"
  rm -rf /tmp/wrt
  git fetch -q origin +refs/pull/$pr/head:refs/verifier/rt$pr || { echo "  FETCH_FAIL"; return 1; }
  [ "$(git rev-parse refs/verifier/rt$pr)" = "$oid" ] || { echo "  PR_REF_MOVED want=$oid"; return 1; }
  git worktree add -q --detach /tmp/wrt $oid || { echo "  WT_FAIL"; return 1; }
  cd /tmp/wrt || return 1
  [ "$(git cat-file blob $oid:$entry | sha256sum | cut -d' ' -f1)" = "$(sha $entry)" ] || { echo "  ENTRY_NOT_HEAD_BLOB"; return 1; }
  CARGO_TARGET_DIR=/root/workspace/repo-root/.trt cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  [ "$(git rev-parse HEAD)" = "$oid" ] || { echo "  BUILT_AT_WRONG_COMMIT"; return 1; }
  local CB=/root/workspace/repo-root/.trt/release/claim_batch
  [ -x $CB ] || { echo "  NO BINARY -- nothing below is a result"; return 1; }
  echo "  BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
  timeout 1700 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $entry --functions "$fns" > /tmp/crt_$tag.out 2>&1
  local rc=$?
  echo "  rc=$rc PASS=$(grep -ac '^PASS ' /tmp/crt_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/crt_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/crt_$tag.out)"
  [ "$rc" = "0" ] && [ "$(grep -ac '^PASS ' /tmp/crt_$tag.out)" = "0" ] && echo "  !! EXIT 0 OBSERVED NOTHING"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/crt_$tag.out | head -8 | sed 's/^/    V| /' | cut -c1-600
  cd /root/workspace/repo-root
  git worktree remove --force /tmp/wrt >/dev/null 2>&1 || true
  git worktree prune >/dev/null 2>&1 || true
  return 0
}
runc 14657 839097ec20a0ebba7788c5d9a1f81d744e54df31 dag/test/claim/design_ruling_standing_witness.dag \
  "an_unclassified_ruling_row_is_refused_and_a_classified_one_is_not" 657b-enrolled
runc 14668 882db05e786e88360884f5c6bb8a2f56bfd442fb dag/test/claim/record_emission_candidate_set_witness.dag \
  "single_field_ambiguous_literal_emits_the_refusal,multi_field_ambiguous_literal_refuses_with_located_diagnostic" 668a-enrolled
runc 14668 882db05e786e88360884f5c6bb8a2f56bfd442fb dag/test/claim/record_emission_candidate_set_witness.dag \
  "single_field_literal_passes_through_bare_value,multi_field_literal_emits_nominal_struct" 668b-controls
echo DONE_ROUTE
