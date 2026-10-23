#!/usr/bin/env bash
# #14805 dispatch C: corrected other-shape control (B's P3 inserted fn BEFORE imports -> parse error, same as the red plant; void).
# Correct plant: append a trivial compiling fn at the END of broken_outside_source (inside the closing quote) => fixture still declares its module,
# census runs, claim must PASS at head (proves head FAIL under P2-plant was the NOT-RUNNABLE census, not any edit).
set -u
REPO=/tmp/c4v/disp2
[ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD_SHA=dfe7025c3903e98f988530e523f0a47e945faf19
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver805c 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver805c
echo 51539607552 > /sys/fs/cgroup/ver805c/memory.max
RB=$(cat /sys/fs/cgroup/ver805c/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver805c memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14805
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14805/head:refs/verifier/f14805c || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14805c)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED got=$(git rev-parse refs/verifier/f14805c) want=$HEAD_SHA -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t14805/release/claim_batch
[ -x $CB ] || { echo NO_BINARY_BUILDING; rm -rf /tmp/w805gb3; git worktree add -q --detach /tmp/w805gb3 $HEAD_SHA && cd /tmp/w805gb3 && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2; cd $REPO; }
[ -x $CB ] || { echo "NO_BINARY -- nothing below is a result"; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA CB_SHA=$(sha $CB) (expect 7dedb0e848f4... if warm)"
FF=dag/test/claim/fleet_fan_mint_seal_witness.dag
rm -rf /tmp/w805ch
git worktree add -q --detach /tmp/w805ch $HEAD_SHA || { echo WTH_FAIL; exit 9; }
python3 - /tmp/w805ch/$FF <<'PY' || exit 9
import sys
p=sys.argv[1]; s=open(p).read()
BS=chr(92)
anchor='r.no_such_field + undefined_name }'+BS+'n'+chr(34)
assert s.count(anchor)==1, f"tail-anchor={s.count(anchor)}"
ins='fn harmless_c() -> Int { 1 }'+BS+'n'
open(p,'w').write(s.replace(anchor, 'r.no_such_field + undefined_name }'+BS+'n'+ins+chr(34)))
print("  plant APPEND-at-end applied (module decl intact, extra fn compiles alongside)")
PY
cd /tmp/w805ch || exit 9
timeout 850 $CB --claim-run --source-root dag --source-root src/v2 --entry $FF --functions witness_a_broken_outside_module_is_not_clean > /tmp/w805c_p3.out 2>&1
echo "  [p3fix_hok] rc=$? PASS=$(grep -ac '^PASS ' /tmp/w805c_p3.out) FAIL=$(grep -ac '^FAIL ' /tmp/w805c_p3.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/w805c_p3.out) BYTES=$(wc -c < /tmp/w805c_p3.out) $(date -u +%H:%M:%S)Z"
grep -aE '^(PASS|FAIL|UNOBSERVED) ' /tmp/w805c_p3.out | sed 's/^/    V| /' | cut -c1-235
cd $REPO
echo DONE_14805C
