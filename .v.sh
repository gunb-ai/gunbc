cargo build --release --bin gunbc --bin claim_batch 2>&1 | tail -1
fl(){ grep -o '^test fn [a-z_0-9]*' $1 | awk '{print $3}' | paste -sd,; }
for W in dag/test/claim/live_deploy/live_deploy_retract_run_witness.dag dag/test/claim/roadmap/attempt_system_unit_launch_witness.dag dag/test/claim/deployed_tree_remote_witness.dag; do
echo "== $W"
./target/release/claim_batch --source-root dag --source-root src/v2 --entry $W --functions $(fl $W) 2>&1 | grep -E '^(FAIL)|rror|PASS the_srv1_retract' | cut -c1-300
done
echo DONE
