#!/usr/bin/env bash
echo "host=$(hostname)"; echo "nproc=$(nproc)"
free -m | sed 's/^/free: /'
echo "cgroup_v2_max=$(cat /sys/fs/cgroup/memory.max 2>/dev/null || echo none)"
echo "cgroup_v1_max=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes 2>/dev/null || echo none)"
echo "cgroup_current=$(cat /sys/fs/cgroup/memory.current 2>/dev/null || echo none)"
echo "rustc=$(rustc -vV | head -3 | tr '\n' ' ')"
echo "cc_env=${CARGO_BUILD_JOBS:-unset} jobs_env CARGO_NET_OFFLINE=${CARGO_NET_OFFLINE:-unset}"
ls /root 2>/dev/null | tr '\n' ' '; echo
du -sh /root/sd503_target 2>/dev/null || echo "sd503_target GONE (stateless confirmed)"
