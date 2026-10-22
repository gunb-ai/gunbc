#!/usr/bin/env bash
# #14692 @ 779fd98addff — d1b: manual exit children WITH the dag root (output policy resolves),
# proving the exit split directly: good rc=0 PASS; closure-error rc=1 FAIL line;
# mixed (ok_fn + broken_fn same entry) rc=1 with PASS and FAIL lines; missing entry rc=2 refusal.
set -u
cd /root/workspace/repo-root 2>/dev/null || cd /tmp/c4v/pr14575 || exit 9
git remote set-url origin https://github.com/gunb-ai/gunbc 2>/dev/null || true
git fetch -q origin +refs/heads/session/nimble-badger-800:refs/heads/session/nimble-badger-800 2>/dev/null || true
git checkout -q -f -B session/nimble-badger-800 origin/session/nimble-badger-800 2>/dev/null || true
OID=779fd98addfff08aed3de2e269a14b393437c7a5
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver692b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver692b
echo 51539607552 > /sys/fs/cgroup/ver692b/memory.max
RB=$(cat /sys/fs/cgroup/ver692b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver692b memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver692b/cgroup.procs; echo $BASHPID >&11
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
git fetch -q --unshallow origin 2>/dev/null || true
git worktree prune >/dev/null 2>&1 || true
sha() { sha256sum "$1" | cut -d' ' -f1; }
git fetch -q origin +refs/pull/14692/head:refs/verifier/692b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/692b)" = "$OID" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/692b) want=$OID -- STOP"; exit 9; }
git worktree remove --force /tmp/w692b 2>/dev/null || true; git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w692b; git worktree add --detach /tmp/w692b $OID 2>&1 | tail -2 | sed 's/^/  WT| /'
[ -d /tmp/w692b ] || { echo WT_FAIL; exit 9; }
cd /tmp/w692b || exit 9
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t692
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
[ "$(git rev-parse HEAD)" = "$OID" ] || { echo "BUILT_AT_WRONG_COMMIT"; exit 9; }
CB=/root/workspace/repo-root/.t692/release/claim_batch
[ -x $CB ] || { echo "NO BINARY"; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
mkdir -p target/release && cp -f $CB target/release/claim_batch
D=target/g692b; rm -rf $D; mkdir -p $D
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
timeout 600 target/release/claim_batch --claim-run --source-root dag --source-root target/g692b --entry target/g692b/good_entry.dag --function good_fn > /tmp/r692_good.out 2>&1; echo "  [good] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r692_good.out) FAIL=$(grep -ac '^FAIL ' /tmp/r692_good.out)"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/r692_good.out | sed 's/^/    O| /'
timeout 600 target/release/claim_batch --claim-run --source-root dag --source-root target/g692b --entry target/g692b/bad_entry.dag --function bad_fn > /tmp/r692_bad.out 2>&1; echo "  [closure-error] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r692_bad.out) FAIL=$(grep -ac '^FAIL ' /tmp/r692_bad.out)"; grep -aE '^(PASS|FAIL|UNOBSERVED) |closure diagnostics' /tmp/r692_bad.out | head -4 | sed 's/^/    O| /' | cut -c1-200
timeout 600 target/release/claim_batch --claim-run --source-root dag --source-root target/g692b --entry target/g692b/mixed_entry.dag --functions ok_fn,broken_fn > /tmp/r692_mix.out 2>&1; echo "  [mixed] rc=$? PASS=$(grep -ac '^PASS ' /tmp/r692_mix.out) FAIL=$(grep -ac '^FAIL ' /tmp/r692_mix.out)"; grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/r692_mix.out | sed 's/^/    O| /' | cut -c1-200
timeout 600 target/release/claim_batch --claim-run --source-root dag --source-root target/g692b --entry target/g692b/no_such_file.dag --function whatever_fn > /tmp/r692_miss.out 2>&1; echo "  [missing-entry] rc=$?"; grep -aE 'UNOBSERVED|refus' /tmp/r692_miss.out | head -3 | sed 's/^/    O| /' | cut -c1-200
rm -rf target/g692b
git status --porcelain | head -3
echo "DONE_14692_D1B $(date -u +%H:%M:%S)Z"
