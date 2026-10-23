#!/usr/bin/env bash
# #14872 cheap confirmation: print the LITERAL cost-partition verdict state word per side,
# plus the out_of_span / sum_exclusive / parent-sum on the same line. Manager asked for the literal
# "Reconciled" (head) vs "Refused"/OverAttributed (base); the full 2x2 already ran.
set -u
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=81b185ec82fc2fb41544f47db2fe008eea546e5e
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver872v 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver872v
echo 51539607552 > /sys/fs/cgroup/ver872v/memory.max
RB=$(cat /sys/fs/cgroup/ver872v/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver872v memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14872/head:refs/verifier/f14872v || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14872v)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14872
H=/tmp/w872vh; B=/tmp/w872vb
git worktree prune >/dev/null 2>&1 || true; rm -rf $H $B
git worktree add -q --force --detach $H $HEAD_SHA || { echo WTH_FAIL; exit 9; }
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
(cd $H && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_H_FAIL; exit 9; }
mkdir -p $H/target/release && cp -f $CARGO_TARGET_DIR/release/claim_batch $H/target/release/claim_batch
echo "HEAD_BIN sha=$(sha $H/target/release/claim_batch) (expect 9e8b8c8cf76a... deterministic)"
git -C $REPO show $HEAD_SHA:src/v1/stage0/src/cli_run.rs > /tmp/cb872v_head.rs 2>/dev/null || git -C $H show HEAD:src/v1/stage0/src/cli_run.rs > /tmp/cb872v_head.rs
(cd $B && git apply -R -p1 $REPO/tools/revert872.patch) || { echo APPLY_R_FAIL; exit 9; }
cmp -s $B/src/v1/stage0/src/cli_run.rs /tmp/cb872v_head.rs && { echo "REVERSE_APPLY_NOOP -- STOP"; exit 9; }
grep -q 'pub out_of_span' $B/src/v1/stage0/src/cli_run.rs && { echo "REVERSE_APPLY_INCOMPLETE -- STOP"; exit 9; }
echo REVERSE_APPLY_OK
(cd $B && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo BUILD_B_FAIL; exit 9; }
mkdir -p $B/target/release && cp -f $CARGO_TARGET_DIR/release/claim_batch $B/target/release/claim_batch
echo "BASE_BIN sha=$(sha $B/target/release/claim_batch) (expect 4b4a9b1681a1...)"
SCF=dag/test/claim/string_family_cast.dag
vline() { # tag driver tree
  local tag=$1 drv=$2 tree=$3
  cd $tree || exit 9
  timeout 900 $drv --claim-run --source-root dag --source-root src/v2 --entry $SCF --functions string_family_cast_holds > /tmp/w872v_$tag.out 2>&1
  local cp=$(grep -aE '\[cost-partition\]' /tmp/w872v_$tag.out | tail -1)
  echo "  [$tag] claim=$(grep -aE '^PASS ' /tmp/w872v_$tag.out | head -1 | cut -c1-40)"
  echo "    verdict_state=$(printf '%s' "$cp" | grep -oE '\"verdict\":\{\"state\":\"[A-Za-z]+\"' | head -1)"
  echo "    cause_kind=$(printf '%s' "$cp" | grep -oE '\"kind\":\"[A-Za-z]+\"' | tr '\n' ' ')"
  echo "    out_of_span=$(printf '%s' "$cp" | grep -oE '\"out_of_span\":\{[^}]*\}' | head -1)"
  echo "    sum_exclusive=$(printf '%s' "$cp" | grep -oE '\"sum_exclusive_nanos\":[0-9]+' | head -1) parent=$(printf '%s' "$cp" | grep -oE '\"parent_span_nanos\":[0-9]+' | head -1)"
  cd $REPO || exit 9
}
vline v_head $H/target/release/claim_batch $H
vline v_base $B/target/release/claim_batch $B
echo DONE_14872V
