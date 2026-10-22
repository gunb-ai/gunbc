set -u
cd /root/workspace/repo-root 2>/dev/null || true
# Create a memory-bounded cgroup subtree and run claim_batch inside it (root, cgroup2).
mkdir -p /sys/fs/cgroup/verifier680 2>&1 | head -1
echo 4294967296 > /sys/fs/cgroup/verifier680/memory.max 2>&1 && echo "memory.max set" || echo "memory.max FAIL"
CAT=/root/workspace/repo-root/.t680p/release/claim_batch
ls -la $CAT 2>/dev/null || { echo CB_MISS; ls /root/workspace/repo-root/.t680f/release/claim_batch 2>/dev/null; }
# launch into the cgroup: exec via bash -c with self-move
( echo $BASHPID > /sys/fs/cgroup/verifier680/cgroup.procs 2>/dev/null; cat /proc/self/cgroup; cd /root/workspace/ver680p && exec $CAT --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry src/v2/test/claim/normalize/service_spine.dag --functions "ss_value_normalizes_to_the_canonical_spine_holds" ) > /tmp/cgrpfix.out 2>&1; rc=$?
echo "rc=$rc PASS=$(grep -ac '^PASS ' /tmp/cgrpfix.out) FAIL=$(grep -ac '^FAIL ' /tmp/cgrpfix.out)"
grep -a 'HostBudgetUnreadable' /tmp/cgrpfix.out | head -1 | cut -c1-120
grep -aE '^(PASS|FAIL) ' /tmp/cgrpfix.out | head -2 | cut -c1-120
