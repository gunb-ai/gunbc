set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14601 @ e52a7deff67 (merges #14692 @ 779fd98) -- parse-gate witness re-verify at printed head:
# manual child RED + control, witness claims by name (outer wet), no-op mutation of
# refuse_unparseable_module_sources, byte-exact restore. ALL claim_batch runs run inside this
# dispatch's bounded cgroup subtree (memory.max=48G, read-back asserted != max).
# Plus: duplicate-field probe through the fold claims in src/v2/test/claim/local_repo_wet_terminal.dag.
OID=e52a7deff67d4b4988b46e1168d7a8342d9bf47f  # gh pr view 14601 --json headRefOid
echo "HEADSHA_from_gh=$OID"
echo "PWD=$(pwd) RUNNER_HEAD=$(git rev-parse HEAD | cut -c1-12) START $(date -u +%H:%M:%S)Z"
# ---- cgroup wrapper (the whole dispatch lives in the bounded subtree) ----
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/verA601 2>/dev/null || true
mkdir -p /sys/fs/cgroup/verA601 || { echo "CGROUP_MKDIR_FAIL"; exit 9; }
echo 51539607552 > /sys/fs/cgroup/verA601/memory.max
RB=$(cat /sys/fs/cgroup/verA601/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/verA601 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/verA601/cgroup.procs
echo $BASHPID >&11
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w601c
git fetch -q origin +refs/pull/14601/head:refs/verifier/14601c 2>&1 | tail -1
git rev-parse -q --verify refs/verifier/14601c >/dev/null || { echo "FETCH_FAILED"; exit 9; }
[ "$(git rev-parse refs/verifier/14601c)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/14601c) want=$OID -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w601c $OID || { echo "WORKTREE_ADD_FAILED"; exit 9; }
WT=/tmp/w601c
[ "$(git -C $WT rev-parse HEAD)" = "$OID" ] || { echo "WORKTREE_NOT_AT_HEAD"; exit 9; }
sha() { sha256sum "$1" | cut -d' ' -f1; }
CLI=src/v1/stage0/src/cli_run.rs
WIT=dag/test/claim/claim_batch_parse_gate_witness.dag
cd $WT || exit 9
CLI_SHA_HEAD=$(git cat-file blob $OID:$CLI | sha256sum | cut -d' ' -f1)
WIT_SHA_HEAD=$(git cat-file blob $OID:$WIT | sha256sum | cut -d' ' -f1)
[ "$(sha $CLI)" = "$CLI_SHA_HEAD" ] || { echo "CLI_RUN_NOT_HEAD_BLOB -- STOP"; exit 9; }
[ "$(sha $WIT)" = "$WIT_SHA_HEAD" ] || { echo "WITNESS_NOT_HEAD_BLOB -- STOP"; exit 9; }
echo "STAGED_OK worktree files == head blobs (cli_run.rs ${CLI_SHA_HEAD:0:12}, witness ${WIT_SHA_HEAD:0:12})"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t14601c CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t14601c/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z inside worktree"
mkdir -p $WT/target/release && cp -f $CB $WT/target/release/claim_batch
echo "child binary staged at \$WT/target/release/claim_batch"

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
  ( cd $WT && timeout 600 ./target/release/claim_batch --claim-run "${args[@]}" ) > /tmp/c601_$label.out 2>&1
  local rc=$?
  echo "  rc=$rc output=$(wc -c < /tmp/c601_$label.out)B"
  echo "  --- combined output (capped 40L):"; sed 's/^/    O| /' /tmp/c601_$label.out | head -40 | cut -c1-200
  echo "  counters: PASS=$(grep -ac '^PASS ' /tmp/c601_$label.out) FAIL=$(grep -ac '^FAIL ' /tmp/c601_$label.out) refusal=$(grep -ac 'module index refused' /tmp/c601_$label.out) located47=$(grep -ac 'parse_gate_lib.dag:4:7' /tmp/c601_$label.out) planted_path=$(grep -ac 'parse-gate-manual-' /tmp/c601_$label.out)"
  rm -rf "$dir"
  return 0
}
witness_claims() { # label
  local label=$1
  echo "-------- [outer-$label] witness claims by name, wet $(date -u +%H:%M:%S)Z"
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --source-root src/v1 --entry $WIT --functions "$RED,$GRN" > /tmp/c601_wit_$label.out 2>&1
  local rc=$?
  if [ "$rc" = "124" ]; then echo "  rc=124 TIMEOUT -- EXCLUDED, not a result"; return 0; fi
  echo "  rc=$rc output=$(wc -c < /tmp/c601_wit_$label.out)B"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c601_wit_$label.out | sed 's/^/    V| /' | cut -c1-190
  grep -aE '^(PASS|FAIL|UNOBSERVED) '"$RED"'|^(PASS|FAIL|UNOBSERVED) '"$GRN" /tmp/c601_wit_$label.out | wc -l | xargs echo "  verdicts_by_name:"
  return 0
}

echo "=== A. BASELINE child runs (process behaviour the witness pins) ==="
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
[ "$(sha $CLI)" = "$CLI_SHA_HEAD" ] && echo "  RESTORED_OK sha256==head blob" || { echo "  RESTORE_MISMATCH -- STOP, tree left mutated"; exit 9; }
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
cp -f $CB $WT/target/release/claim_batch
echo "  rebuilt restored $(date -u +%H:%M:%S)Z CB_SHA=$(sha $CB | cut -c1-12)"
witness_claims rest
manual_child rest-red unparsable

echo "=== E. DUPLICATE-FIELD PROBE (manager substrate question) ==="
LCW=src/v2/test/claim/local_repo_wet_terminal.dag
# names read from the file at run time
grep -oE 'test fn [a-z_0-9]+\(' $LCW | sed 's/test fn //;s/($//;s/(//' | head -12 | paste -sd, - > /tmp/c601_foldnames.txt
FOLDN=$(cat /tmp/c601_foldnames.txt)
echo "  fold claim names: $FOLDN"
LWT=src/v2/workflow/local_repo_wet_terminal.dag
git cat-file blob HEAD:$LWT > /tmp/lrw_orig.dag
python3 - <<'PYEOF'
t=open('/tmp/lrw_orig.dag').read()
i=t.find('an_unparsable_imported_module_refuses_with_exit_1_and_no_verdict_line', t.find('WetScheduledClaim'))
assert i>0
j=t.find('expectation: ExpectedToHold {}', i)
assert j>0 and j-i<500, "anchor not in row"
t2=t[:j]+'expectation: ExpectedToHold {}\n      '+t[j:]
open('/tmp/dup_probe.dag','w').write(t2)
print("dup probe: added 1 duplicate expectation field:", t2.count('expectation:')-t.count('expectation:')==1)
PYEOF
cp /tmp/dup_probe.dag $LWT
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry $LCW --functions "$FOLDN" > /tmp/c601_dup.out 2>&1
DUPRC=$?
echo "  dup fold-claims rc=$DUPRC"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/c601_dup.out | sed 's/^/    V| /' | cut -c1-180
grep -aiE 'duplicate|refus|error|panic' /tmp/c601_dup.out | grep -avE '^(PASS|FAIL|UNOBSERVED)' | head -6 | sed 's/^/    D| /' | cut -c1-180
git cat-file blob HEAD:$LWT > $LWT
[ "$(sha $LWT)" = "$(git cat-file blob HEAD:$LWT | sha256sum | cut -d' ' -f1)" ] && echo "  dup-probe restore OK" || echo "  DUP_RESTORE_CHECK_FAIL"
echo DONE_14601_C
cd /root/workspace/repo-root
git worktree remove --force $WT >/dev/null 2>&1 || true
git worktree prune >/dev/null 2>&1 || true
