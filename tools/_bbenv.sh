set -u
cd /root/workspace/repo-root 2>/dev/null || true
echo "BUILDBUDDY_RUN_ID=[${BUILDBUDDY_RUN_ID:-UNSET}]"
echo "env grep BB|RUN:"; env | grep -iE 'buildbuddy|BB_|RUN_ID|BAZEL' | head -6 | sed 's/^/    E| /'
cat /sys/fs/cgroup/memory.max 2>&1 | sed 's/^/root-memory.max=/'
cat /sys/fs/cgroup/cgroup.subtree_control 2>&1 | sed 's/^/subtree=/'
