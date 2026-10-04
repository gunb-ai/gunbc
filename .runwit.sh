set -e
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
G=./target/release/gunbc
for f in dag/test/claim/runner/runner_host_medium_witness_test.dag dag/test/claim/runner/runner_throughput_qualification_witness_test.dag dag/test/claim/machine_intake/mtcollins1_census_image_witness_test.dag dag/test/claim/machine_intake/mtcollins1_census_medium_readback_witness_test.dag dag/test/claim/xorriso_path_list_witness_test.dag dag/test/claim/machine_intake/mtcollins1_boot_run_witness_test.dag; do
  for t in $(grep -oE '^test fn [a-z0-9_]+' $f | awk '{print $3}'); do
    out=$($G run --source-root dag --source-root src/v2 --entry $f --function $t 2>&1 | tail -2 | tr '\n' ' ' | cut -c1-300)
    echo "$(basename $f .dag)::$t => $out"
  done
done
