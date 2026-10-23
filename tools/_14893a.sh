#!/usr/bin/env bash
# #14893 RED/control (manager spec). One exact-head OUTER shape is impossible here: the partition
# is computed IN-PROCESS by the driver, so behavior follows the BINARY. Three drivers, all claim
# arms on the HEAD tree:
#  H  = head build                          -> CONTROL: claims rc=0, partition Reconciled
#  P  = head + #14872 cli_run.rs reverse-applied (planted over-attribution), gate ON
#                                           -> rc=1 with claim PASS, Refused/OverAttributed
#  PG = head + plant + THIS PR's gating diff reverse-applied ("base")
#                                           -> rc=0, Refused printed, line continues (the defect)
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=794a10d4ae612da1794d2e9949ba36faf50b1002
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893a 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893a
echo 51539607552 > /sys/fs/cgroup/ver893a/memory.max
RB=$(cat /sys/fs/cgroup/ver893a/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893a memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893a || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893a)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14893a) -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893
mk() { # tag [reverse-patch...]
  local tag=$1; shift
  local T=/tmp/w893_$tag
  git worktree prune >/dev/null 2>&1 || true; rm -rf $T
  git worktree add -q --force --detach $T $HEAD_SHA || { echo "WT_FAIL $tag"; exit 9; }
  for p in "$@"; do
    (cd $T && git apply -R -p1 $REPO/tools/$p) || { echo "APPLY_R_FAIL $p on $tag -- STOP"; exit 9; }
  done
  (cd $T && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1) || { echo "BUILD_FAIL $tag -- STOP"; exit 9; }
  cp -f $CARGO_TARGET_DIR/release/claim_batch $T/claim_batch
  echo "BIN[$tag] sha=$(sha $T/claim_batch) $(date -u +%H:%M:%S)Z"
}
echo "=== builds: $(date -u +%H:%M:%S)Z"
mk h
mk p plant893.patch
mk pg plant893.patch gate893.patch
# guards: plants changed behavior surface; PG lacks the gate symbols
grep -q 'pub out_of_span' /tmp/w893_p/src/v1/stage0/src/cli_run.rs && { echo "PLANT_NOOP -- STOP"; exit 9; }
grep -q 'pub out_of_span' /tmp/w893_pg/src/v1/stage0/src/cli_run.rs && { echo "PLANT_NOOP pg -- STOP"; exit 9; }
grep -q 'fn partition_exit_flags' /tmp/w893_pg/src/v1/stage0/src/bin/claim_batch.rs && { echo "GATE_NOT_REVERSED pg -- STOP"; exit 9; }
grep -q 'fn partition_exit_flags' /tmp/w893_p/src/v1/stage0/src/bin/claim_batch.rs || { echo "GATE_MISSING p -- STOP"; exit 9; }
echo PLANTS_OK
claim() { # tag bin claim
  local tag=$1 drv=$2 fn=$3
  cd /tmp/w893_h || exit 9
  timeout 900 $drv --claim-run --source-root dag --source-root src/v2 --entry dag/test/claim/string_family_cast.dag --functions $fn > /tmp/w893_$tag.out 2>&1
  local rc=$?
  local cp=$(grep -aE '\[cost-partition\]' /tmp/w893_$tag.out | tail -1)
  echo "  [$tag $fn] rc=$rc PASS=$(grep -ac "^PASS $fn$" /tmp/w893_$tag.out) FAIL=$(grep -ac "^FAIL $fn$" /tmp/w893_$tag.out) verdict=$(printf '%s' "$cp" | grep -oE '\"verdict\":\{\"state\":\"[A-Za-z]+\"' | head -1) kind=$(printf '%s' "$cp" | grep -oE '\"kind\":\"[A-Za-z]+\"' | head -1) oos=$(printf '%s' "$cp" | grep -oc 'out_of_span') BYTES=$(wc -c < /tmp/w893_$tag.out)"
  cd $REPO || exit 9
}
for fn in string_family_cast_holds unicode_ident_xid_grounding_holds; do
  echo "=== arms $fn: $(date -u +%H:%M:%S)Z"
  claim H_h  /tmp/w893_h/claim_batch $fn
  claim P_p  /tmp/w893_p/claim_batch $fn
  claim PG_pg /tmp/w893_pg/claim_batch $fn
done
echo DONE_14893A
