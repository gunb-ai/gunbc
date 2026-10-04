set -u
export PAGER=cat GIT_PAGER=cat
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch --bin v1_src_dag_parse 2>&1 | tail -1
mkdir -p /sys/fs/cgroup/gunbcrun && echo +memory > /sys/fs/cgroup/cgroup.subtree_control && echo 23622320128 > /sys/fs/cgroup/gunbcrun/memory.max && echo $BASHPID > /sys/fs/cgroup/gunbcrun/cgroup.procs
run(){ echo "=== $1"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $1 --functions $2 > /tmp/cb.out 2>&1; rc=$?; grep -E "^FAIL" /tmp/cb.out; echo pass=$(grep -c "^PASS" /tmp/cb.out); if [ $rc -ne 0 ]; then grep -v "^\[" /tmp/cb.out | grep -vE "^\s*(✓|◐|◷)|^(PASS|FAIL)|^claim_batch|memory-cgroup" | tail -10; fi; echo rc=$rc; }
cp dag/gunbc/live_deploy/role_singleton_convergence.dag /tmp/c.bak
echo "=== MUTANT B: multiplicity deduplicated (two-hosts and misplaced dropped)"
sed -i 's/  let two_hosts = ids |> flat_map(d => if count(/  let two_hosts_unused = ids |> flat_map(d => if count(/; s/  let misplaced = ids |> flat_map(/  let misplaced_unused = ids |> flat_map(/; s/concat(concat(unobservable, two_hosts), concat(misplaced, two_on_one))/concat(concat(unobservable, []), concat([], two_on_one))/' dag/gunbc/live_deploy/role_singleton_convergence.dag; grep -c 'two_hosts_unused' dag/gunbc/live_deploy/role_singleton_convergence.dag
run dag/test/claim/deployment_risk_witness_test.dag the_same_deployment_on_two_hosts_refuses,a_deployment_targeted_at_srv2_but_realized_on_srv1_refuses,exactly_one_complete_realization_of_the_desired_holder_on_its_target_is_a_noop
cp /tmp/c.bak dag/gunbc/live_deploy/role_singleton_convergence.dag
echo "=== MUTANT D: every desired refusal read as no-prod"
sed -i 's/    ProdRoleHolderUnrealized { deployment: _ } => false/    ProdRoleHolderUnrealized { deployment: _ } => true/; s/    ProdRoleHolderAmbiguous { deployment: _, count: _ } => false/    ProdRoleHolderAmbiguous { deployment: _, count: _ } => true/' dag/gunbc/live_deploy/role_singleton_convergence.dag; grep -c 'ProdRoleHolderUnrealized { deployment: _ } => true' dag/gunbc/live_deploy/role_singleton_convergence.dag
run dag/test/claim/deployment_risk_witness_test.dag an_unrealized_or_ambiguous_holder_refuses_as_desired_side_in_empty_and_occupied_worlds
cp /tmp/c.bak dag/gunbc/live_deploy/role_singleton_convergence.dag
echo "=== MUTANT E: holding pinned to srv1-live"
sed -i 's/      if holder.instance.deployment == instance.deployment { HoldsProdRoleSingletons/      if (instance.instance_id as String) == "srv1-live" { HoldsProdRoleSingletons/' dag/gunbc/live_deploy/spec.dag
run dag/test/claim/deployment_risk_witness_test.dag moving_the_prod_role_moves_every_role_singleton_member_to_the_new_holders_spec,the_real_selection_places_every_singleton_on_its_holder_and_converges_as_noop
cp /tmp/c.bak dag/gunbc/live_deploy/role_singleton_convergence.dag
echo "=== MUTANT F: adoption admits any agreeing spec"
sed -i 's/      else if desired_is_this_spec \&\& (spec.target.ssh_host as String) == (host as String) { admitted }/      else if true { admitted }/' dag/gunbc/live_deploy/role_singleton_convergence.dag; grep -c 'else if true { admitted }' dag/gunbc/live_deploy/role_singleton_convergence.dag
run dag/test/claim/deployment_risk_witness_test.dag an_unmarked_holder_on_its_own_target_is_adopted_and_the_plan_writes_the_marker
