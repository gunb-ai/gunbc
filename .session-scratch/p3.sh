echo "P3 host=$(hostname) t=$(date +%H:%M:%S)"
ps aux --sort=-%cpu | head -15
echo ---LOCKS---
ls -la /root/workspace/cargo-target-persist/ 2>&1 | head -8
ls -la /root/workspace/cargo-target-persist/.package-lock.json 2>/dev/null
ls /root/workspace/cargo-target-persist/debug/claim_batch 2>&1
echo ---TREES---
ls -d /root/workspace/* 2>/dev/null
echo P3_DONE
