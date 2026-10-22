#!/usr/bin/env bash
# #14793 @ 92d35c4f0db7dd19bf3181fc9e507ee67e434a93 (base ed5930a0cc834b1c7d89b1c300cff8d680467687)
# Arms: head-built binary runs witness 2 claims at HEAD (both PASS) and at BASE (RED FAIL, control PASS);
# head ledger 10/10 by name + wire 18/18 by name; delta-vs-9d34b030 confirmed (delete-only, checked locally too).
set -u
cd /root/workspace/repo-root 2>/dev/null || true
REPO=/root/workspace/repo-root
[ -d /tmp/c4v/pr14575 ] && REPO=/tmp/c4v/pr14575
cd $REPO || exit 9
echo "REPO=$REPO"
HEAD=92d35c4f0db7dd19bf3181fc9e507ee67e434a93
BASE=ed5930a0cc834b1c7d89b1c300cff8d680467687
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver793 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver793
echo 51539607552 > /sys/fs/cgroup/ver793/memory.max
RB=$(cat /sys/fs/cgroup/ver793/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver793 memory.max=$RB (read-back, bounded)"
exec 11>>/sys/fs/cgroup/ver793/cgroup.procs 2>/dev/null && echo $BASHPID >&11 || echo "CGROUP_JOIN_BUSY (recorded, proceeding)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
export CARGO_TARGET_DIR=/root/workspace/repo-root/.t793
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
rm -rf /tmp/w793h /tmp/w793b
git fetch -q origin +refs/pull/14793/head:refs/verifier/f793 || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f793)" = "$HEAD" ] || { echo "PR_REF_MOVED fetched=$(git rev-parse refs/verifier/f793) want=$HEAD -- STOP"; exit 9; }
git worktree add -q --detach /tmp/w793h $HEAD || { echo WT_FAIL; exit 9; }
cd /tmp/w793h || exit 9
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo WT_NOT_AT_HEAD; exit 9; }
WI=dag/test/claim/observation_unrepresentable_reason_witness.dag
LE=src/v2/test/claim/workflow/floor_terminal_ledger.dag
LW=src/v2/test/claim/workflow/floor_terminal_ledger_wire.dag
for f in $WI $LE $LW; do [ "$(git cat-file blob $HEAD:$f | sha256sum | cut -d' ' -f1)" = "$(sha $f)" ] || { echo "NOT_HEAD_BLOB $f -- STOP"; exit 9; }; done
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
CB=/root/workspace/repo-root/.t793/release/claim_batch
[ -x $CB ] || { echo "NO BINARY -- nothing below is a result"; exit 9; }
[ "$(git rev-parse HEAD)" = "$HEAD" ] || { echo BUILT_AT_WRONG_COMMIT; exit 9; }
echo "BUILD_COMMIT=$(git rev-parse HEAD) CB_SHA=$(sha $CB) built $(date -u +%H:%M:%S)Z"
NW="an_empty_literal_reason_is_refused,a_nonempty_str_reason_is_accepted_by_the_retyped_field"
NLE=$(grep -oE '^test fn [A-Za-z_0-9]+' $LE | sed 's/^test fn //' | paste -sd, -)
NLW=$(grep -oE '^test fn [A-Za-z_0-9]+' $LW | sed 's/^test fn //' | paste -sd, -)
echo "LEDGER_COUNT=$(echo $NLE | tr ',' '\n' | wc -l) (10) WIRE_COUNT=$(echo $NLW | tr ',' '\n' | wc -l) (18)"
echo "=== HEAD witness 2 by name: $(date -u +%H:%M:%S)Z"
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $WI --functions "$NW" > /tmp/f793_head_wi.out 2>&1; echo "[head WI] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f793_head_wi.out) FAIL=$(grep -ac '^FAIL ' /tmp/f793_head_wi.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f793_head_wi.out) bytes=$(wc -c < /tmp/f793_head_wi.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f793_head_wi.out | sed 's/^/  V| /' | cut -c1-200
echo "=== BASE witness 2 by name (same binary; expect RED FAIL, control PASS): $(date -u +%H:%M:%S)Z"
git worktree add -q --detach /tmp/w793b $BASE || { echo BASE_WT_FAIL; exit 9; }
cd /tmp/w793b || exit 9
[ "$(git -C /tmp/w793b rev-parse HEAD)" = "$BASE" ] || { echo BASE_NOT_AT_BASE; exit 9; }
[ "$(git cat-file blob $BASE:$WI | sha256sum | cut -d' ' -f1)" = "$(sha $WI)" ] && echo "  base witness blob == head (delete-only delta applies to witness file? see note)" || echo "  base witness blob DIFFERS from head (expected: it has the dropped probe + no retype? check below)"
git diff --stat $BASE $HEAD -- $WI | sed 's/^/  DELTA-WI| /' | head -3
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $WI --functions "$NW" > /tmp/f793_base_wi.out 2>&1; echo "[base WI] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f793_base_wi.out) FAIL=$(grep -ac '^FAIL ' /tmp/f793_base_wi.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f793_base_wi.out) bytes=$(wc -c < /tmp/f793_base_wi.out)"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/f793_base_wi.out | sed 's/^/  V| /' | cut -c1-200
echo "=== HEAD ledger 10 + wire 18 by name: $(date -u +%H:%M:%S)Z"
cd /tmp/w793h || exit 9
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $LE --functions "$NLE" > /tmp/f793_head_le.out 2>&1; echo "[head LE] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f793_head_le.out) FAIL=$(grep -ac '^FAIL ' /tmp/f793_head_le.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f793_head_le.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/f793_head_le.out | head -4 | sed 's/^/  V| /' | cut -c1-200
timeout 1200 $CB --claim-run --source-root dag --source-root src/v2 --entry $LW --functions "$NLW" > /tmp/f793_head_lw.out 2>&1; echo "[head LW] rc=$? PASS=$(grep -ac '^PASS ' /tmp/f793_head_lw.out) FAIL=$(grep -ac '^FAIL ' /tmp/f793_head_lw.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/f793_head_lw.out)"
grep -aE '^(FAIL|UNOBSERVED) ' /tmp/f793_head_lw.out | head -4 | sed 's/^/  V| /' | cut -c1-200
git status --porcelain | head -3
echo DONE_793 $(date -u +%H:%M:%S)Z
