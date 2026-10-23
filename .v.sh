cargo build --release --bin gunbc --bin claim_batch 2>&1 | tail -1
for w in live_deploy/emit roadmap/attempt_system_unit_launch_witness live_deploy/live_deploy_apply_plan_witness live_deploy/live_deploy_apply_run_witness; do
 f=dag/test/claim/$w.dag; echo W $w
 F=$(grep -o '^test fn [a-z_0-9]*' $f | awk '{print $3}' | grep -v 'writes_through_the_launcher_to_bin_and_source_hooks_are_refused\|a_wide_request_naming_a_root_exit_hook_is_refused_by_the_launcher' | paste -sd,)
 ./target/release/claim_batch --wet --source-root dag --source-root src/v2 --entry $f --functions "$F" 2>&1 | grep -aE '^FAIL|rror' | head -5
done
echo ALLDONE
