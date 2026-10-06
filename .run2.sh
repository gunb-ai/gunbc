set -uo pipefail
echo "TREE_DIFF_SHA=$(git diff HEAD | sha256sum | cut -c1-16)"
cargo build --release -p v1-compiler --bin claim_batch --target-dir /tmp/cb-tgt 2>&1 | tail -2
B=/tmp/cb-tgt/release/claim_batch
sha256sum $B
id -u; cat /proc/self/cgroup; mount | grep cgroup | head -3
CG=/sys/fs/cgroup/cb$$
if mkdir $CG 2>/dev/null; then echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null; echo 20000000000 > $CG/memory.max && echo $$ > $CG/cgroup.procs && echo "CGROUP bound $(cat $CG/memory.max)"; else echo "CGROUP mkdir failed"; fi
free -g | head -2
for f in "$@"; do fns=$(grep -oP '^test fn \K\w+' $f | paste -sd,); echo "=== WITNESS $f"; $B --claim-run --hermetic --source-root dag --source-root src/v2 --entry $f --functions "$fns" > /tmp/out.txt 2>&1; rc=$?; grep -E "^(PASS|FAIL|claim_batch: resolve failed)|error|^\[witness\]|refus" /tmp/out.txt | cut -c1-400 | head -80; [ $rc -ge 2 ] && tail -c 3000 /tmp/out.txt; echo "PEAK $(cat $CG/memory.peak 2>/dev/null)"; echo "=== EXIT $f $rc"; done
