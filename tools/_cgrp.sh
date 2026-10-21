set -u
cd /root/workspace/repo-root 2>/dev/null || true
echo "id=$(id -u) kernel=$(uname -r)"
echo "--- mounts"; mount | grep -i cgroup | head -5; echo "(end mounts)"
echo "--- proc cgroup"; cat /proc/self/cgroup
echo "--- ls /sys/fs/cgroup"; ls /sys/fs/cgroup 2>&1 | head -5
CB=/root/workspace/repo-root/.t680p/release/claim_batch
echo "CB exists: $([ -x $CB ] && echo yes || echo no)"
