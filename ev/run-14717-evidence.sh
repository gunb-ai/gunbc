set -u
# Evidence at exact head d5c71a581ba for PR #14717 option A.
# Mutation REDs:
#  1: observe/boot event-sha ref pointed at needs.build.outputs  -> all three claims FAIL
#  2: apply ref loses its || github.sha fallback                  -> all three claims FAIL
#  3 (NEW, vs #14690's substring form): shared_job_if replaced by a literal that CONTAINS the
#     required text for both excluded modes but is not the exact derivation from the mode list
#     -> namecheap+printer FAIL, boot PASS (boot never reads shared_job_if), so the exact
#        derivation conjunct is what fails: #14690's substring claim would stay green here.
B=/tmp/privtarget14717/release/claim_batch
R="--wet --source-root dag --source-root src/v2"
W=dag/gunbc/fleet/fleet_converge_workflow.dag
three(){
 $B $R --entry dag/test/claim/machine_intake/mtcollins1_boot_run_witness.dag --functions another_expected_revision_cannot_change_a_byte_the_boot_job_executes 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | sed 's/^/boot /' | cut -c1-140
 for f in dag/test/claim/namecheap_workflow_witness.dag dag/test/claim/printer_workflow_binding_witness.dag; do
   $B $R --entry $f --functions trusted_code_cannot_be_replaced_by_dispatch_revision 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | sed "s|^|$(basename $f .dag) |" | cut -c1-140
 done
}
echo "== HEAD: the three"
three
echo "== CONTROL: sibling claim, unmutated (expect PASS)"
$B $R --entry dag/test/claim/printer_workflow_binding_witness.dag --functions printer_and_main_workspace_modes_remain_dispatchable 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-140

echo "== MUTATION 1: event-sha ref pointed at needs.build.outputs (expect three FAIL)"
k='data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_event_sha)'
grep -qF "$k" $W && sed -i "s#$k#data fleet_converge_event_sha_checkout_ref_expr: String = interpolate(expression: fleet_converge_build_revision_output)#" $W && three
git checkout -q -- $W

echo "== MUTATION 2: apply ref loses its || github.sha fallback (expect three FAIL)"
k='data fleet_converge_checkout_ref: Expression = LogicalOr { left: fleet_converge_build_revision_output, right: fleet_converge_event_sha }'
grep -qF "$k" $W && sed -i "s#$k#data fleet_converge_checkout_ref: Expression = fleet_converge_build_revision_output#" $W && three
git checkout -q -- $W

echo "== MUTATION 3 (NEW): shared_job_if replaced by a literal that CONTAINS #14690's required text for both modes but is not the exact derivation (expect namecheap+printer FAIL, boot PASS)"
k="data fleet_converge_shared_job_if: String = fleet_converge_any_mode_step_if(modes: filter(fleet_converge_workflow_modes, mode => fleet_converge_mode_job_id(mode: mode) == fleet_converge_shared_job_id))"
grep -qF "$k" $W && sed -i "s#$k#data fleet_converge_shared_job_if: String = \"github.event.inputs.mode != 'namecheap_observe' \\&\\& github.event.inputs.mode != 'printer'\"#" $W && echo "mutation applied: $(grep -c \"github.event.inputs.mode != 'namecheap_observe'\" $W) occurrences of the substring #14690 demanded" && three
git checkout -q -- $W
echo "== restored: $(git diff --stat | wc -l) files dirty"
