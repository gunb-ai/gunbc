#!/usr/bin/env bash
# Wet probe: manager asked "run every claim wet". The 23 migrated files have 0 rows in
# src/v2/test/claim/local_repo_wet_terminal.dag at head (grep evidence printed below).
# Probe: run 2 claims through --wet. If they give verdicts, wet IS the lane and I must re-run all 358 wet.
# If rc=2 zero-verdict (my standing rule for non-enrolled claims), the N/A is OBSERVED, not asserted.
set -u
REPO=/tmp/c4v/disp2; [ -d $REPO ] || REPO=/root/workspace/repo-root
cd $REPO || exit 9
HEAD_SHA=dfe7025c3903e98f988530e523f0a47e945faf19
[ -d /sys/fs/cgroup ] || { echo "NO_CGROUP2 -- STOP"; exit 9; }
echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null || true
rm -rf /sys/fs/cgroup/ver805w 2>/dev/null || true; mkdir -p /sys/fs/cgroup/ver805w
echo 51539607552 > /sys/fs/cgroup/ver805w/memory.max
RB=$(cat /sys/fs/cgroup/ver805w/memory.max)
{ [ "$RB" != "max" ] && [ -n "$RB" ]; } || { echo "MEMORY_BOUND_READBACK_FAIL got=$RB -- STOP"; exit 9; }
echo "CGROUP_WRAPPER=/sys/fs/cgroup/ver805w memory.max=$RB (read-back, bounded)"
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=/root/workspace/repo-root/.t14805
sha() { sha256sum "$1" | cut -d' ' -f1; }
git worktree prune >/dev/null 2>&1 || true
git fetch -q origin +refs/pull/14805/head:refs/verifier/f14805w || { echo FETCH_FAIL; exit 9; }
[ "$(git rev-parse refs/verifier/f14805w)" = "$HEAD_SHA" ] || { echo "PR_REF_MOVED -- STOP"; exit 9; }
CB=/root/workspace/repo-root/.t14805/release/claim_batch
if [ ! -x $CB ]; then
  rm -rf /tmp/w805gwb; git worktree add -q --detach /tmp/w805gwb $HEAD_SHA || { echo WT_FAIL; exit 9; }
  cd /tmp/w805gwb && cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1; cd $REPO || exit 9
fi
[ -x $CB ] || { echo "NO_BINARY -- STOP"; exit 9; }
echo "BUILD_COMMIT=$HEAD_SHA CB_SHA=$(sha $CB)"
WT=/tmp/w805wet; rm -rf $WT; git worktree add -q --detach $WT $HEAD_SHA || { echo WT_FAIL; exit 9; }
echo "=== wet-schedule grep evidence (expect 0 across all 23):"
tot=0
while read -r f; do
  b=$(basename "$f" .dag)
  h=$(grep -c "$b" $WT/src/v2/test/claim/local_repo_wet_terminal.dag 2>/dev/null || true)
  tot=$((tot+h))
done < tools/w14805_files.txt
echo "  total wet-schedule mentions of the 23 witness files = $tot"
probe() { # entry fn...
  cd $WT || exit 9
  timeout 600 $CB --claim-run --wet --source-root dag --source-root src/v2 --entry "$1" --functions "$2" > /tmp/wet_$2.out 2>&1
  echo "  [wet $2] rc=$? PASS=$(grep -ac '^PASS ' /tmp/wet_$2.out) FAIL=$(grep -ac '^FAIL ' /tmp/wet_$2.out) UNOBS=$(grep -ac '^UNOBSERVED ' /tmp/wet_$2.out) BYTES=$(wc -c < /tmp/wet_$2.out)"
  head -6 /tmp/wet_$2.out | sed 's/^/    O| /' | cut -c1-200
  cd $REPO || exit 9
}
echo "=== wet probes:"
probe dag/test/claim/fleet_fan_mint_seal_witness.dag witness_a_legal_outside_module_compiles_clean
probe dag/test/claim/sole_constructor_type_head_exposure_witness.dag sole_constructor_type_head_is_not_exposed
echo DONE_WETPROBE
