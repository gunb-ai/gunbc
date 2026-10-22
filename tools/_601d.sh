#!/usr/bin/env bash
# #14601 @ c2b863a6fc7b6155671793e5c1d18c13098b93a0 (main merge onto e52a7de) -- manager-scoped
# re-verify: re-run parse-gate RED + control + 2 witness claims by name at the printed head.
# Static delta audit (merge parents, 2 resolved files, schedule rows, stall file) done locally.
set -u
OID=c2b863a6fc7b28bf86d26054de2fb704e49867e1  # gh pr view 14601 --json headRefOid
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
# cold runner: make sure we can reach the PR ref regardless of shallow sync
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
echo "HEADSHA_expected=$OID"
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verD601 2>/dev/null || true
mkdir -p /sys/fs/cgroup/verD601 || { echo "CGROUP_MKDIR_FAIL"; exit 9; }
echo 51539607552 > /sys/fs/cgroup/verD601/memory.max
RB=$(cat /sys/fs/cgroup/verD601/memory.max)
[ "$RB" != "max" ] || { echo "CGROUP_UNBOUNDED -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verD601 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verD601/cgroup.procs
echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w601d
git fetch -q origin +refs/pull/14601/head:refs/verifier/14601d 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14601d >/dev/null || { echo "FETCH_FAILED"; exit 9; }
[ "$(git rev-parse refs/verifier/14601d)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/14601d) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w601d $OID || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/w601d
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
CLI=src/v1/stage0/src/cli_run.rs
WIT=dag/test/claim/claim_batch_parse_gate_witness.dag
cd $WT || exit 9
CLI_SHA_HEAD=$(git cat-file blob $OID:$CLI | sha256sum | cut -d' ' -f1)
WIT_SHA_HEAD=$(git cat-file blob $OID:$WIT | sha256sum | cut -d' ' -f1)
[ "$(sha $CLI)" = "$CLI_SHA_HEAD" ] || { echo "CLI_RUN_NOT_HEAD_BLOB -- STOP"; exit 9; }
[ "$(sha $WIT)" = "$WIT_SHA_HEAD" ] || { echo "WITNESS_NOT_HEAD_BLOB -- STOP"; exit 9; }
echo "STAGED_OK worktree==head blobs (cli_run ${CLI_SHA_HEAD:0:12}, witness ${WIT_SHA_HEAD:0:12})"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14601d CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t14601d/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z inside worktree"
mkdir -p $WT/target/release && cp -f $CB $WT/target/release/claim_batch

RED=an_unparsable_imported_module_refuses_with_exit_1_and_no_verdict_line
GRN=a_parseable_imported_module_reaches_a_real_pass_verdict

plant() { # $1=dir  $2=lib source tag
  rm -rf "$1"; mkdir -p "$1"
  cat > "$1/parse_gate_entry.dag" <<'DAG'
module probe.parse_gate_entry

import probe.parse_gate_lib { parse_gate_lib_value }

test fn parse_gate_entry_holds() -> Bool {
  parse_gate_lib_value()
}
DAG
  if [ "$2" = unparsable ]; then
    printf 'module probe.parse_gate_lib\n\nfn parse_gate_lib_value() -> Bool {\n  true\n' > "$1/parse_gate_lib.dag"
  else
    printf 'module probe.parse_gate_lib\n\nfn parse_gate_lib_value() -> Bool {\n  true\n}\n' > "$1/parse_gate_lib.dag"
  fi
  echo "  planted $1 lib=$2 bytes=$(wc -c < "$1/parse_gate_lib.dag")"
}
manual_child() { # label libtag [with-dag]
  local label=$1 tag=$2 dagroot=${3:-no}
  local dir=$WT/target/parse-gate-manual-$tag
  plant "$dir" $tag
  local args=(--source-root "$dir" --entry "$dir/parse_gate_entry.dag" --function parse_gate_entry_holds)
  [ "$dagroot" = with-dag ] && args=(--source-root dag --source-root "$dir" --entry "$dir/parse_gate_entry.dag" --function parse_gate_entry_holds)
  echo "-------- [child-$label] roots=$( [ "$dagroot" = with-dag ] && echo 'dag+planted' || echo 'planted-only' ) $(date -u +%H:%M:%S)Z"
  ( cd $WT && timeout 600 ./target/release/claim_batch --claim-run "${args[@]}" ) > /tmp/d601_$label.out 2>&1
  local rc=$?
  echo "  rc=$rc output=$(wc -c < /tmp/d601_$label.out)B"
  echo "  --- combined output (capped 40L):"; sed 's/^/    O| /' /tmp/d601_$label.out | head -40 | cut -c1-200
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/d601_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/d601_$label.out) refusal=$(grep -ac 'module index refused' /tmp/d601_$label.out) located47=$(grep -ac 'parse_gate_lib.dag:4:7' /tmp/d601_$label.out) planted_path=$(grep -ac 'parse-gate-manual-' /tmp/d601_$label.out)"
  rm -rf "$dir"
  return 0
}

echo "=== A. child runs at head: RED (unparsable) + CONTROL (parseable, dag+planted) ==="
manual_child base-red unparsable
manual_child base-ctl parseable with-dag

echo "=== B. witness claims by name, wet, ENROLLED route (dag + src/v2) ==="
timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$RED,$GRN" > /tmp/d601_wit.out 2>&1
rc=$?
echo "  rc=$rc output=$(wc -c < /tmp/d601_wit.out)B"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/d601_wit.out | sed 's/^/    V| /' | cut -c1-190
n=$(grep -aE '^(PASS|FAIL|UNOBSERVED) '"$RED"'|^(PASS|FAIL|UNOBSERVED) '"$GRN" /tmp/d601_wit.out | wc -l)
echo "  verdicts_by_name=$n/2"
[ "$n" = "2" ] || echo "  !! ZERO-VERDICT-GUARD: fewer than 2 verdicts observed -- NOT a result"
git -C $WT status --porcelain | head -3
echo "DONE_601D $(date -u +%H:%M:%S)Z"
