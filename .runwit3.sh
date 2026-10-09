cargo build --release -q --bin claim_batch -p v1-compiler; echo build=$?; echo head=$(git rev-parse --short HEAD)
mkdir -p /sys/fs/cgroup/gunbcrun && echo "+memory" > /sys/fs/cgroup/cgroup.subtree_control
echo 21474836480 > /sys/fs/cgroup/gunbcrun/memory.max && echo $BASHPID > /sys/fs/cgroup/gunbcrun/cgroup.procs || { echo "REFUSED: cgroup"; exit 9; }
echo cgroup=bound
F=dag/gunbc/machine_intake/mtcollins1_boot_run.dag
W=dag/test/claim/runner/runner_host_medium_witness_test.dag
run() { fns=$(grep -oE '^test fn [a-z0-9_]+' $1 | awk '{print $3}' | paste -sd,); GITHUB_SHA=$(git rev-parse HEAD) ./target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $1 --functions "$fns" > /tmp/o 2>&1; echo "rc=$? pass=$(grep -c '^PASS' /tmp/o) fail=$(grep -cE '^(FAIL|RED|NOT)' /tmp/o)"; grep -E '^(FAIL|RED|NOT)|did not hold' /tmp/o | head -3 | cut -c1-200; }
echo "== baseline runner_host_medium"; run $W
echo "== baseline boot_run"; run dag/test/claim/machine_intake/mtcollins1_boot_run_witness_test.dag
echo "== baseline acceptance_matrix"; run dag/test/claim/machine_intake/mtcollins1_boot_acceptance_matrix_test.dag
echo "== baseline census_image"; run dag/test/claim/machine_intake/mtcollins1_census_image_witness_test.dag
echo "== baseline route"; run dag/test/claim/runner/runner_throughput_qualification_witness_test.dag
echo "== baseline probe"; run dag/test/claim/provisioning/seed_artifact_fetch_forged_probe_witness_test.dag
echo "== baseline xorriso"; run dag/test/claim/xorriso_path_list_witness_test.dag
echo "== baseline census_readback"; run dag/test/claim/machine_intake/mtcollins1_census_medium_readback_witness_test.dag
cp $F /tmp/F.orig
awk '/^fn actuation_after_watch\(/{inf=1} inf && /runner_host: observed.runner_host,/{sub(/observed.runner_host/,"none"); inf=0} {print}' /tmp/F.orig > $F; echo "== MUTANT1 actuation_after_watch runner_host:none diff=$(diff /tmp/F.orig $F | grep -c '^>')"; run $W
awk '/^fn actuation_with_outcome\(/{inf=1} inf && /runner_host: act.runner_host,/{sub(/act.runner_host/,"none"); inf=0} {print}' /tmp/F.orig > $F; echo "== MUTANT2 actuation_with_outcome runner_host:none diff=$(diff /tmp/F.orig $F | grep -c '^>')"; run $W
cp /tmp/F.orig $F
