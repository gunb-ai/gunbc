set -u
git fetch -q origin scratch/keen-owl-704-bootref-merge; H=$(git rev-parse FETCH_HEAD); echo "FETCHED $H (expected 6e7134997d41c70b3705c5ff4451adce65cf777f)"
git worktree add -q --detach /tmp/co17r $H && cd /tmp/co17r && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
W=dag/gunbc/fleet/fleet_converge_workflow.dag
three(){ $B $R --entry dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag --functions another_expected_revision_cannot_change_a_byte_the_boot_job_executes 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-160
 for f in dag/test/claim/namecheap_workflow_witness.dag dag/test/claim/printer_workflow_binding_witness.dag; do $B $R --entry $f --functions trusted_code_cannot_be_replaced_by_dispatch_revision 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-160 | sed "s#^#$(basename $f) #"; done; }
echo "== HEAD: the three"; three
echo "== MUTATION 1: boot ref pointed at needs.build.outputs (expect boot+namecheap FAIL)"
k='data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_event_sha)'
grep -qF "$k" $W && sed -i "s#$k#data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_build_revision_output)#" $W && three
git checkout -q -- $W
echo "== MUTATION 2: apply ref loses its || github.sha fallback (expect FAIL)"
k='data fleet_converge_checkout_ref: Expression = LogicalOr { left: fleet_converge_build_revision_output, right: fleet_converge_event_sha }'
grep -qF "$k" $W && sed -i "s#$k#data fleet_converge_checkout_ref: Expression = fleet_converge_build_revision_output#" $W && three
