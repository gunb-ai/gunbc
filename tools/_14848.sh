#!/usr/bin/env bash
# #14848 verification: claim_batch ExitFailure code 2 -> UNOBSERVED/exit 2.
# Manager spec: ONE exact-head OUTER binary on both sides; base = same tree with the claim_batch.rs
# change reverse-applied (git apply -R, guarded). code-2 claim: exit 2 side at head, exit 1 side at
# base; code-1 and held controls PASS both. Plus: 3 new wet schedule rows actually execute the witness.
# The witness spawns <toplevel>/target/release/claim_batch as the CHILD under test -> each tree gets
# its own child binary from the shared-cache cargo build (sha printed; expected eec337e414bc head /
# 748d3e9dd0be base per the PR body).
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=242ed6058876e430b8b6a2e4d7685a47dbe6381e
WIT=dag/test/claim/claim_batch_exit_two_unobserved_witness.dag
C2=an_exit_failure_with_code_2_exits_2_and_prints_unobserved
C1=an_exit_failure_with_code_1_still_exits_1_and_prints_fail
HELD=a_held_claim_exits_0_and_prints_pass
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver848 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver848
echo 51539607552 > /sys/fs/cgroup/ver848/memory.max
RB=$(cat /sys/fs/cgroup/ver848/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver848 memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14848/head:refs/verifier/f14848 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14848)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14848) -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14848
H=/tmp/w848h; B=/tmp/w848b
git worktree prune >/dev/null 2>&1 || true; rm -rf $H $B
git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
echo "=== build head tree: $(date -u +%H:%M:%S)Z"
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1)
mkdir -p $H/target/release && cp -f $CARGO_TARGET_DIR/release/claim_batch $H/target/release/claim_batch || exit 9
echo "HEAD_BIN sha=$(sha $H/target/release/claim_batch) (expect eec337e414bc... if deterministic)"
echo "=== reverse-apply claim_batch.rs on base tree: $(date -u +%H:%M:%S)Z"
git show refs/verifier/f14848:src/v1/stage0/src/bin/claim_batch.rs > /tmp/cb_head.rs 2>/dev/null || git -C $REPO show $HEAD_SHA:src/v1/stage0/src/bin/claim_batch.rs > /tmp/cb_head.rs
(cd $B && git apply -R -p1 $REPO/tools/revert848.patch) || { echo "APPLY_R_FAIL -- STOP"; exit 9; }
cmp -s $B/src/v1/stage0/src/bin/claim_batch.rs /tmp/cb_head.rs && { echo "REVERSE_APPLY_NOOP -- STOP"; exit 9; }
grep -q 'code: 2, reason' $B/src/v1/stage0/src/bin/claim_batch.rs && { echo "REVERSE_APPLY_INCOMPLETE -- STOP"; exit 9; }
echo REVERSE_APPLY_OK
echo "=== build base tree (shared cache): $(date -u +%H:%M:%S)Z"
(cd $B && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1)
mkdir -p $B/target/release && cp -f $CARGO_TARGET_DIR/release/claim_batch $B/target/release/claim_batch || exit 9
echo "BASE_BIN sha=$(sha $B/target/release/claim_batch) (expect 748d3e9dd0be...)"
[ "$(sha $H/target/release/claim_batch)" != "$(sha $B/target/release/claim_batch)" ] || { echo "BINS_IDENTICAL -- STOP"; exit 9; }
CBH=$H/target/release/claim_batch   # ONE head-built outer driver, both sides
echo "OUTER_DRIVER sha=$(sha $CBH) TREE=h AND TREE=b"
run() { # tag tree claim [extra...]
  local tag=$1 tree=$2; shift 2
  local claim=$1; shift
  cd $tree || exit 9
  timeout 900 $CBH --claim-run "$@" --source-root dag --source-root src/v2 --entry $WIT --functions $claim > /tmp/w848_$tag.out 2>&1
  local rc=$?
  echo "  [$tag] rc=$rc PASS=$(grep -ac '^PASS ' /tmp/w848_$tag.out) FAIL=$(grep -ac '^FAIL ' /tmp/w848_$tag.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/w848_$tag.out) BYTES=$(wc -c < /tmp/w848_$tag.out) $(date -u +%H:%M:%S)Z"
  grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/w848_$tag.out | head -3 | sed 's/^/    V| /' | cut -c1-230
  cd $REPO || exit 9
}
echo "=== head wet arms (expect PASS x3): $(date -u +%H:%M:%S)Z"
run h_wet_c2 $H $C2 --wet
run h_wet_c1 $H $C1 --wet
run h_wet_held $H $HELD --wet
echo "=== base wet arms (expect FAIL on c2, PASS c1+held): $(date -u +%H:%M:%S)Z"
run b_wet_c2 $B $C2 --wet
run b_wet_c1 $B $C1 --wet
run b_wet_held $B $HELD --wet
echo "=== wet-gate control: non-wet run of an enrolled claim (expect rc=2 zero-verdict => wet rows are load-bearing): $(date -u +%H:%M:%S)Z"
run h_nowet_c2 $H $C2
echo DONE_14848
