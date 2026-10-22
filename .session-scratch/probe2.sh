echo "PROBE2 host=$(hostname) cwd=$(pwd)"
echo "TREE_HEAD=$(git rev-parse HEAD 2>/dev/null)"
ls -la /root/workspace/cargo-target-persist/debug/claim_batch 2>&1 | head -2
ls /root/workspace/ 2>&1
echo "CARGO=$(which cargo) $(cargo --version 2>&1)"
[ -x /root/workspace/cargo-target-persist/debug/claim_batch ] && /root/workspace/cargo-target-persist/debug/claim_batch --help 2>&1 | head -5
echo PROBE2_DONE
