set -u
git fetch -q origin main scratch/keen-owl-704-bootref-merge c4-boot-ref-witnesses
M=6e7134997d41c70b3705c5ff4451adce65cf777f; MAIN=$(git rev-parse $M^2 2>/dev/null || true); echo "MERGE $M PARENTS $(git rev-parse $M^1 $M^2 | tr '\n' ' ')"
P=dag/test/claim/printer_workflow_binding_witness.dag; R="--wet --source-root dag --source-root src/v2"
for rev in $M $(git rev-parse $M^1) $(git rev-parse $M^2); do
 d=/tmp/co-$rev; git worktree add -q --detach $d $rev; ( cd $d
 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error" | head -3
 echo "== REV $rev BIN $(sha256sum target/release/claim_batch | cut -c1-16)"
 target/release/claim_batch $R --entry $P --functions printer_and_main_workspace_modes_remain_dispatchable 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)'
 if [ $rev = $M ]; then W=dag/gunbc/fleet/fleet_converge_workflow.dag
  k='data fleet_converge_checkout_ref: Expression = LogicalOr { left: fleet_converge_build_revision_output, right: fleet_converge_event_sha }'
  grep -qF "$k" $W && sed -i "s#$k#data fleet_converge_checkout_ref: Expression = fleet_converge_build_revision_output#" $W && echo "== MUTATION 2 applied"
  target/release/claim_batch $R --entry dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag --functions another_expected_revision_cannot_change_a_byte_the_boot_job_executes 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)'
  for f in dag/test/claim/namecheap_workflow_witness.dag $P; do target/release/claim_batch $R --entry $f --functions trusted_code_cannot_be_replaced_by_dispatch_revision 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | sed "s#^#$(basename $f) #"; done
  git checkout -q -- $W; fi ); done
