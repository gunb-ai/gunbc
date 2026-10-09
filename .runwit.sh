cargo build --release -p v1-compiler --bin claim_batch >/dev/null 2>&1; echo build=$?; git rev-parse --short HEAD
for f in dag/test/claim/machine_intake/mtcollins1_boot_acceptance_matrix_test.dag dag/test/claim/runner/runner_host_medium_witness_test.dag dag/test/claim/machine_intake/mtcollins1_boot_run_witness_test.dag dag/test/claim/runner/runner_throughput_qualification_witness_test.dag dag/test/claim/machine_intake/mtcollins1_census_image_witness_test.dag dag/test/claim/srv3/srv3_seeded_install_media_witness_test.dag dag/test/claim/provisioning/seed_artifact_fetch_forged_probe_witness_test.dag; do
  fns=$(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}' | paste -sd,)
  ./target/release/claim_batch --hermetic --source-root dag --source-root src/v2 --entry $f --functions "$fns" > /tmp/o 2>&1; rc=$?
  echo "== $f rc=$rc pass=$(grep -ciE '\bPASS|held' /tmp/o)"; grep -iE "FAIL|did not hold|refus" /tmp/o | grep -v "^\[pre-entry\]" | head -4 | cut -c1-400; tail -2 /tmp/o | cut -c1-300
done
