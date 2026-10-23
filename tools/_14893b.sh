#!/usr/bin/env bash
# #14893 dispatch B: (1) unit tests incl a_refused_partition_joins_the_batch_exit +
# exit_composition_is_failed_over_unobserved_over_clean; (2) NoSpans->2 mutation probe on the
# gate mapping; (3) rfm-rung-census: the retired RFM row's new AtCeiling standing must resolve --
# census case live_roster_every_read_row_is_admitted held, no unread row, no refusal, at head;
# base = same tree, RFM row restored to BelowCeiling (reverse-apply 794a10d4ae) for comparison.
set -u
REPO=/root/workspace/repo-root; [ -d /tmp/c4v/disp2 ] && REPO=/tmp/c4v/disp2
cd $REPO || exit 9
HEAD_SHA=794a10d4ae612da1794d2e9949ba36faf50b1002
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver893b 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver893b
echo 51539607552 > /sys/fs/cgroup/ver893b/memory.max
RB=$(cat /sys/fs/cgroup/ver893b/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver893b memory.max=$RB (read-back, bounded)"
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14893/head:refs/verifier/f14893b || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14893b)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14893
T=/tmp/w893b_h
git worktree prune >/dev/null 2>&1 || true; rm -rf $T
git worktree add -q --force --detach $T $HEAD_SHA || { echo WT_FAIL; exit 9; }
echo "=== unit tests (claim_batch bin target): $(date -u +%H:%M:%S)Z"
(cd $T && timeout 2400 cargo test -p v1-compiler --bin claim_batch 2>&1) | grep -aE 'running [0-9]+ test|test result|^test a_refused|^test exit_composition|FAILED|panicked|error\[' | sed 's/^/  UT| /' | cut -c1-170
echo "=== NoSpans mapping probe: mutation flips NoSpans arm to (false,true); expect exactly a_refused_partition_joins... FAILED: $(date -u +%H:%M:%S)Z"
python3 - $T/src/v1/stage0/src/bin/claim_batch.rs <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
a='''            cause: CostAccountingRefusal::NoSpans,
        } => (true, false),'''
assert s.count(a)==1, f"anchor {s.count(a)}"
open(p,'w').write(s.replace(a,'''            cause: CostAccountingRefusal::NoSpans,
        } => (false, true),'''))
print("  MUT NoSpans->(false,true) applied")
PY
(cd $T && timeout 2400 cargo test -p v1-compiler --bin claim_batch 2>&1) | grep -aE 'running [0-9]+ test|test result|^test a_refused|^test exit_composition|FAILED|panicked|error\[' | sed 's/^/  M1| /' | cut -c1-170
git -C $T checkout -q -- src/v1/stage0/src/bin/claim_batch.rs
echo "=== census (head): gunbc test //gunbc/instruments:rfm-rung-census via gunbc bin: $(date -u +%H:%M:%S)Z"
(cd $T && cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1)
GUNBC=$CARGO_TARGET_DIR/release/gunbc; echo "GUNBC_SHA=$(sha $GUNBC)"
(cd $T && timeout 1200 $GUNBC test //gunbc/instruments:rfm-rung-census 2>/dev/null) | grep -aE 'case |roster |a_refusal_prints|Held|held|NOT|refus' | sed 's/^/  CN| /' | cut -c1-200
echo "=== census (base, RFM row BelowCeiling restored): $(date -u +%H:%M:%S)Z"
B=/tmp/w893b_b
git worktree prune >/dev/null 2>&1 || true; rm -rf $B
git worktree add -q --force --detach $B $HEAD_SHA || { echo WTB_FAIL; exit 9; }
cp $REPO/tools/stage893/a_refusal_prints_and_the_line_continues.base.dag $B/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || { echo "BASE_BLOB_FAIL -- STOP"; exit 9; }
grep -q 'BelowCeiling' $B/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag || { echo "RESTORE_NOOP -- STOP"; exit 9; }
echo BASE_RESTORE_OK
(cd $B && timeout 1200 $GUNBC test //gunbc/instruments:rfm-rung-census 2>/dev/null) | grep -aE 'case |roster |a_refusal_prints|Held|held|NOT|refus' | sed 's/^/  CB| /' | cut -c1-200
echo "=== static: retired row AtCeiling + no BelowCeiling in head row; rung_climb import gone:"
grep -c 'AtCeiling' $T/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag | sed 's/^/  ATCEILING_COUNT=/'
grep -c 'BelowCeiling\|ClimbableButUnbuilt' $T/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag | sed 's/^/  BELOW_COUNT=/'
grep -c 'gunbc#14893' $T/dag/gunbc/recurring_failure_mode/a_refusal_prints_and_the_line_continues.dag | sed 's/^/  RETIRED_MENTION=/'
echo DONE_14893B
