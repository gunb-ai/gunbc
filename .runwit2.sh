cargo build --release -q --bin claim_batch -p v1-compiler; echo build=$?; echo head=$(git rev-parse --short HEAD)
mkdir -p /sys/fs/cgroup/gunbcrun && echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control
echo 21474836480 > /sys/fs/cgroup/gunbcrun/memory.max && echo $BASHPID > /sys/fs/cgroup/gunbcrun/cgroup.procs || { echo "REFUSED: cgroup"; exit 9; }
echo cgroup=bound
for f in dag/test/claim/machine_intake/mtcollins1_boot_acceptance_matrix_test.dag dag/test/claim/runner/runner_host_medium_witness_test.dag dag/test/claim/machine_intake/mtcollins1_boot_run_witness_test.dag dag/test/claim/runner/runner_throughput_qualification_witness_test.dag dag/test/claim/provisioning/seed_artifact_fetch_forged_probe_witness_test.dag dag/test/claim/machine_intake/mtcollins1_census_image_witness_test.dag; do
  n=$(grep -cE '^test fn ' $f); fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,)
  GITHUB_SHA=$(git rev-parse HEAD) ./target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $f --functions "$fns" > /tmp/o 2>&1; rc=$?
  echo "== $f fns=$n rc=$rc"; grep -iE "held|pass|fail|did not|summary|[0-9]+/[0-9]+" /tmp/o | grep -v "^\[pre-entry\]\|typecheck" | tail -6 | cut -c1-400
done
