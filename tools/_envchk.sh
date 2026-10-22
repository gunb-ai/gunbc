set -u
cd /root/workspace/repo-root 2>/dev/null || true
echo "cgroup v2: $(cat /sys/fs/cgroup/memory.max 2>&1) $(cat /sys/fs/cgroup/memory.high 2>&1)"
echo "cgroup v1: $(cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>&1 | head -1)"
CB=/root/workspace/.t680p/release/claim_batch
[ -x $CB ] || CB=$(ls /root/workspace/*/release/claim_batch 2>/dev/null | head -1)
echo "CB=$CB"
cd /root/workspace/ver680p 2>/dev/null || { echo NO_WT; exit 0; }
export GUNBC_MEMORY_BUDGET_BYTES=4294967296
timeout 900 $CB --claim-run --source-root dag --source-root src/v2 --source-root src/v1 --entry src/v2/test/claim/normalize/service_spine.dag --functions "ss_value_normalizes_to_the_canonical_spine_holds" > /tmp/envchk.out 2>&1; rc=$?
echo "with-env rc=$rc PASS=$(grep -ac '^PASS ' /tmp/envchk.out) FAIL=$(grep -ac '^FAIL ' /tmp/envchk.out)"
grep -a 'HostBudgetUnreadable\|declared-unverified' /tmp/envchk.out | head -2 | cut -c1-160
grep -aE '^(PASS|FAIL) ' /tmp/envchk.out | cut -c1-120
