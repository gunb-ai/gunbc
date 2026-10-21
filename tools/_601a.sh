set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14601 @ eaa56d3ad0e0 -- parse-gate witness: manual RED + control on the CHILD, the witness
# claims by name on the OUTER (wet), then the no-op mutation of refuse_unparseable_module_sources,
# then byte-exact restore. The ONLY build happens inside the head worktree; source hashed around it.
OID=eaa56d3ad0e07738ecc10c03c33dd77eed5b592e  # gh pr view 14601 --json headRefOid
echo "HEADSHA_from_gh=$OID"
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) START $(date -u +%H:%M:%S)Z"
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w601
# dedicated ref, not FETCH_HEAD: a concurrent dispatch's fetch would race FETCH_HEAD
git fetch -q origin +refs/pull/14601/head:refs/verifier/14601 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14601 >/dev/null || { echo "FETCH_FAILED"; exit 9; }
[ "$(git rev-parse refs/verifier/14601)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/14601) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w601 $OID || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/w601
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
CLI=src/v1/stage0/src/cli_run.rs
WIT=dag/test/claim/claim_batch_parse_gate_witness.dag
cd $WT || exit 9
[ "$(sha $CLI)" = "7c36840416a033e5d105925ea3a41d716631eb8fff6d07ac51b2ee8396f0f848" ] || { echo "CLI_RUN_NOT_HEAD_BLOB -- STOP"; exit 9; }
[ "$(sha $WIT)" = "d50400ca3b1dd0a9957c2c1c13b67c7e9baeb0cd074db219f49bfbc504a9a5e3" ] || { echo "WITNESS_NOT_HEAD_BLOB -- STOP"; exit 9; }
cp -f $CLI /root/workspace/repo-root/tools/stage/cli_run_601_head.rs 2>/dev/null || { mkdir -p /root/workspace/repo-root/tools/stage; cp -f $CLI /root/workspace/repo-root/tools/stage/cli_run_601_head.rs; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14601 CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t14601/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "CB_SHA=$(sha $CB | cut -c1-12) built $(date -u +%H:%M:%S)Z inside worktree at $(git rev-parse HEAD | cut -c1-12)"
# the witness spawns <git-toplevel>/target/release/claim_batch: toplevel is the worktree
mkdir -p $WT/target/release && cp -f $CB $WT/target/release/claim_batch
echo "child binary staged at target/release/claim_batch sha=$(sha $WT/target/release/claim_batch | cut -c1-12)"

RED=an_unparsable_imported_module_refuses_with_exit_1_and_no_verdict_line
GRN=a_parseable_imported_module_reaches_a_real_pass_verdict
WITNESS=dag/test/claim/claim_batch_parse_gate_witness.dag

plant() { # $1=dir  $2=lib source tag (unparsable|parseable)
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
  ( cd $WT && timeout 600 ./target/release/claim_batch --claim-run "${args[@]}" > /tmp/c601_$label.out 2> /tmp/c601_$label.err ); local rc=$?
  echo "  rc=$rc stdout=$(wc -c < /tmp/c601_$label.out)B stderr=$(wc -c < /tmp/c601_$label.err)B"
  echo "  --- stdout (full, capped 40L):"; sed 's/^/    O| /' /tmp/c601_$label.out | head -40 | cut -c1-200
  echo "  --- stderr head (capped 25L):"; head -25 /tmp/c601_$label.err | sed 's/^/    E| /' | cut -c1-200
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/c601_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/c601_$label.out) refusal=$(grep -ac 'module index refused' /tmp/c601_$label.err) located47=$(grep -ac 'parse_gate_lib.dag:4:7' /tmp/c601_$label.err) planted_path=$(grep -ac 'parse-gate-manual-' /tmp/c601_$label.err)"
  rm -rf "$dir"
  return 0
}
witness_claims() { # label
  local label=$1
  echo "-------- [outer-$label] witness claims by name, wet $(date -u +%H:%M:%S)Z"
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WITNESS --functions "$RED,$GRN" > /tmp/c601_wit_$label.out 2> /tmp/c601_wit_$label.err
  local rc=$?
  if [ "$rc" = "124" ]; then echo "  rc=124 TIMEOUT -- EXCLUDED, not a result"; return 0; fi
  echo "  rc=$rc stdout=$(wc -c < /tmp/c601_wit_$label.out)B stderr=$(wc -c < /tmp/c601_wit_$label.err)B"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c601_wit_$label.out | sed 's/^/    V| /' | cut -c1-190
  grep -aE '^(PASS|FAIL|UNOBSERVED) '"$RED"'|^(PASS|FAIL|UNOBSERVED) '"$GRN" /tmp/c601_wit_$label.out | wc -l | xargs echo "  verdicts_by_name:"
  tail -8 /tmp/c601_wit_$label.err | grep -aE 'error|refused|panic' | sed 's/^/    E| /' | cut -c1-190
  return 0
}

echo "=== A. BASELINE child runs (the process behaviour the witness pins) ==="
manual_child base-red unparsable
manual_child base-ctl parseable with-dag
echo "=== B. BASELINE witness claims (outer, wet, both by name) ==="
witness_claims base

echo "=== C. MUTATION: make refuse_unparseable_module_sources a no-op ==="
grep -c 'if refusals.is_empty() {' $CLI | xargs echo "  anchor count (must be 1):"
grep -n 'fn refuse_unparseable_module_sources' $CLI | sed 's/^/  at /'
sed -i 's/if refusals\.is_empty() {/if true { \/\/ MUTATION no-op gate/' $CLI || { echo "SED_FAILED"; exit 9; }
[ "$(grep -c 'MUTATION no-op gate' $CLI)" = "1" ] || { echo "MUTATION_NOT_APPLIED"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
cp -f $CB $WT/target/release/claim_batch
echo "  rebuilt with mutation $(date -u +%H:%M:%S)Z CB_SHA=$(sha $CB | cut -c1-12)"
manual_child mut-red unparsable
witness_claims mut
echo "=== D. RESTORE byte-exact + rebuild + re-run ==="
git cat-file blob HEAD:$CLI > $CLI
[ "$(sha $CLI)" = "7c36840416a033e5d105925ea3a41d716631eb8fff6d07ac51b2ee8396f0f848" ] && echo "  RESTORED_OK sha256==head blob" || { echo "  RESTORE_MISMATCH -- STOP, tree left mutated"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
cp -f $CB $WT/target/release/claim_batch
echo "  rebuilt restored $(date -u +%H:%M:%S)Z CB_SHA=$(sha $CB | cut -c1-12)"
witness_claims rest
manual_child rest-red unparsable
echo DONE_14601_A
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
