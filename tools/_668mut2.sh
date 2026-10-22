#!/usr/bin/env bash
# #14668 census-targeted suppression mutation: the AmbiguousAnonymousRecordLiteral DIAGNOSTIC
# itself (make_error_node at v1_compiler_emit_rust.rs ~:35126) -> empty vec. Both refusal claims
# (emit-text single + census-count multi) must FAIL; both controls PASS. Restore byte-exact.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=7c97832508ad532bcc996b5ac520b862c43cb33f   # gh-resolved
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver668n 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver668n
echo 51539607552 > /sys/fs/cgroup/ver668n/memory.max
RB=$(cat /sys/fs/cgroup/ver668n/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver668n memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver668n/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=dag/test/claim/record_emission_candidate_set_witness.dag
EMIT=src/v1/stage0/src/v1_compiler_emit_rust.rs
git fetch -q origin +refs/pull/14668/head:refs/verifier/668n || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/668n)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/668n) want=$OID -- STOP"; exit 9; }
rm -rf /tmp/w668n; git worktree add -q --detach /tmp/w668n $OID || { echo WT_FAIL; exit 9; }
cd /tmp/w668n || exit 9
for f in $WIT $EMIT; do
  [ "$(git cat-file blob $OID:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }
done
NAMES=$(grep -oE '^test fn [a-z_0-9]+' $WIT | sed 's/^test fn //' | paste -sd,)
echo "HEADSHA=$OID NAMES_FROM_FILE=$NAMES"
export CARGO_TARGET_DIR=/root/workspace/repo-root/.tb3n
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.tb3n/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB)"
run4() {
  local tag=$1
  timeout 1500 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$NAMES" > /tmp/n668_${tag}.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/n668_${tag}.out) FAIL=$(grep -ac '^FAIL ' /tmp/n668_${tag}.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/n668_${tag}.out)"
  for n in $(echo $NAMES | tr ',' ' '); do
    echo "    $n: PASS=$(grep -ac "^PASS $n\$" /tmp/n668_${tag}.out) FAIL=$(grep -ac "^FAIL $n\$" /tmp/n668_${tag}.out)"
  done
}
cp -f $EMIT /tmp/emitn_head_orig
echo "=== A. baseline (expect 4 PASS):"
run4 base
echo "=== B. MUTATION: kill the AmbiguousAnonymousRecordLiteral diagnostic node:"
BEFORE=$(grep -acF 'Rc::new(vec![crate::v1_std_core::make_error_node(Rc::new(CompilerDiagnostic::AmbiguousAnonymousRecordLiteral {' $EMIT)
echo "  anchor before=$BEFORE (1 expected)"
[ "$BEFORE" = "1" ] || { echo "MUT_ANCHOR_FAIL -- NOT a mutation run"; }
perl -0777 -pi -e 's/Rc::new\(vec!\[crate::v1_std_core::make_error_node\(Rc::new\(CompilerDiagnostic::AmbiguousAnonymousRecordLiteral \{.*?module_name\.clone\(\)\)\)\])/Rc::new(vec![])/s' $EMIT
AFTER=$(grep -acF 'make_error_node(Rc::new(CompilerDiagnostic::AmbiguousAnonymousRecordLiteral' $EMIT)
[ "$AFTER" = "0" ] || { echo "MUT_REPLACE_FAIL after=$AFTER -- NOT a mutation run"; }
echo "  mutated sha=$(sha $EMIT) orig=$(sha /tmp/emitn_head_orig)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
echo "  mut CB_SHA=$(sha $CB)"
run4 mut
echo "=== C. restore byte-exact, rebuild, re-run:"
cp -f /tmp/emitn_head_orig $EMIT
[ "$(sha $EMIT)" = "$(sha /tmp/emitn_head_orig)" ] && echo "  RESTORE_BYTE_EXACT=YES" || echo "  RESTORE_BYTE_EXACT=NO -- STOP"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "  post CB_SHA=$(sha $CB)"
run4 post
git status --porcelain | head -3
echo "DONE_668N $(date -u +%H:%M:%S)Z"
