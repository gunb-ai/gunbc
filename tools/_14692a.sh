#!/usr/bin/env bash
# #14692 (MERGED re-land) @ 779fd98addff — d1: cargo check + head-built binary + 2 wet claims
# by name + manual exit children: missing-only expect rc=2 refusal; closure-only expect rc=1 FAIL
# line; mixed (good fn + bad fn in one entry) expect rc=1 with PASS and FAIL lines.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=779fd98addfff08aed3de2e269a14b393437c7a5   # git ls-remote pull/14692/head
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver692 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver692
echo 51539607552 > /sys/fs/cgroup/ver692/memory.max
RB=$(cat /sys/fs/cgroup/ver692/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver692 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver692/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
WIT=dag/test/claim/claim_batch_unobserved_exit_witness.dag
RED1=an_entry_with_a_closure_error_exits_1_with_a_fail_line
RED2=a_nonexistent_entry_exits_2_with_a_located_refusal
git fetch -q origin +refs/pull/14692/head:refs/verifier/692 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/692)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/692) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w692 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w692; git worktree add --detach /tmp/w692 $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w692 ] || { echo WT_FAIL; exit 9; }
cd /tmp/w692 || exit 9
[ "$(git cat-file blob $OID:$WIT | sha256sum | cut -d' ' -f1)" = "$(sha $WIT)" ] || { echo "NOT_HEAD_BLOB WIT -- STOP"; exit 9; }
grep -aq "test fn $RED1" $WIT && grep -aq "test fn $RED2" $WIT && echo "STAGED_OK names present" || { echo NAMES_MISSING; exit 8; }
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t692
echo "=== cargo check --release -p v1-compiler:"
cargo check --release -p v1-compiler 2>&1 | tail -2
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t692/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
mkdir -p target/release && cp -f $CB target/release/claim_batch
echo "=== A. 2 wet claims by name:"
timeout 1800 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry $WIT --functions "$RED1,$RED2" > /tmp/q692_wet.out 2>&1
echo "  rc=$? PASS=$(grep -ac '^PASS ' /tmp/q692_wet.out) FAIL=$(grep -ac '^FAIL ' /tmp/q692_wet.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/q692_wet.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/q692_wet.out | sed 's/^/    V| /' | cut -c1-200
echo "=== B. manual children (planted entries, planted-only route):"
D=target/g692; rm -rf $D; mkdir -p $D
cat > $D/good_entry.dag <<'DAG'
module probe.good_entry

test fn good_fn() -> Bool {
  true
}
DAG
cat > $D/bad_entry.dag <<'DAG'
module probe.bad_entry

test fn bad_fn() -> Bool {
  no_such_function_anywhere()
}
DAG
cat > $D/mixed_entry.dag <<'DAG'
module probe.mixed_entry

test fn ok_fn() -> Bool {
  true
}

test fn broken_fn() -> Bool {
  no_such_function_anywhere()
}
DAG
cd $D || exit 9
timeout 600 ../../target/release/claim_batch --claim-run --source-root . --entry good_entry.dag --function good_fn > /tmp/q692_c_good.out 2>&1; echo "  [child good] rc=$? $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/q692_c_good.out | head -1)"
timeout 600 ../../target/release/claim_batch --claim-run --source-root . --entry bad_entry.dag --function bad_fn > /tmp/q692_c_bad.out 2>&1; echo "  [child closure-error] rc=$? $(grep -aoE '^(PASS|FAIL|UNOBSERVED)' /tmp/q692_c_bad.out | head -1)"; head -6 /tmp/q692_c_bad.out | sed 's/^/    O| /' | cut -c1-200
timeout 600 ../../target/release/claim_batch --claim-run --source-root . --entry mixed_entry.dag --functions ok_fn,broken_fn > /tmp/q692_c_mix.out 2>&1; echo "  [child mixed] rc=$? PASS=$(grep -ac '^PASS ' /tmp/q692_c_mix.out) FAIL=$(grep -ac '^FAIL ' /tmp/q692_c_mix.out)"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/q692_c_mix.out | sed 's/^/    O| /' | cut -c1-200
timeout 600 ../../target/release/claim_batch --claim-run --source-root . --entry no_such_file.dag --function whatever_fn > /tmp/q692_c_miss.out 2>&1; echo "  [child missing-entry] rc=$?"; head -4 /tmp/q692_c_miss.out | sed 's/^/    O| /' | cut -c1-200
cd /tmp/w692 || exit 9
rm -rf target/g692
git status --porcelain | head -3
echo "DONE_14692_D1 $(date -u +%H:%M:%S)Z"
