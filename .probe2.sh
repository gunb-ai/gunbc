cargo build --release -q --bin claim_batch -p v1-compiler; echo head=$(git rev-parse --short HEAD)
CG=/sys/fs/cgroup/gunbcrun; mkdir -p $CG && echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control; echo 21474836480 > $CG/memory.max && echo $BASHPID > $CG/cgroup.procs || { echo "REFUSED: cgroup"; exit 9; }
f=dag/test/claim/runner/runner_host_medium_witness_test.dag
fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,)
GITHUB_SHA=$(git rev-parse HEAD) ./target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $f --functions "$fns" > /tmp/o 2>&1; echo PROBE_RC=$?
grep -E "error|refus|resolve failed" /tmp/o | sort -u | head -8 | cut -c1-500
