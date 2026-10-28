set -u; MODE=${1:-}
G=/sys/fs/cgroup/kp$$; mkdir -p $G && echo 48G > $G/memory.max && echo $$ > $G/cgroup.procs
git fetch -q origin main scratch/keen-owl-704-printer-merge
H=89e23eee29c8e28e7aa75983af8ce8984c6aef1a
P=dag/test/claim/printer_workflow_binding_witness.dag; W=dag/gunbc/fleet/fleet_converge_workflow.dag
R="--wet --source-root dag --source-root src/v2"
ALL=observer_has_no_fleet_ssh_demand,trusted_code_cannot_be_replaced_by_dispatch_revision,observer_is_a_registered_dispatch_and_run_target,printer_and_main_workspace_modes_remain_dispatchable
ALL=$(grep -aoE '^test fn [a-z0-9_]+' /dev/null; true)
cb(){ target/release/claim_batch $R --entry $P --functions "$1" > /tmp/cb.out 2>&1; echo "EXIT $?"; grep -aE '^(PASS|FAIL|UNOBS)' /tmp/cb.out | cut -c1-500; }
d=/tmp/pp; git worktree add -q --detach $d $H; cd $d
if [ "$MODE" = B ]; then git checkout -q --detach b129ed84b691a129fdf98872967de60c2fbe6ef0 && git -c user.email=x@x -c user.name=x cherry-pick $H >/dev/null && echo PICKED; fi
echo "== MODE $MODE TREE $(git rev-parse HEAD) PARENTS $(git log -1 --format=%P)"
git diff --stat b129ed84b691a129fdf98872967de60c2fbe6ef0 HEAD -- $P | tail -1
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error" | head -3
echo "BIN $(sha256sum target/release/claim_batch | cut -c1-16)"
FNS=$(grep -aoE '^test fn [a-z0-9_]+' $P | sed 's/test fn //' | paste -sd,)
echo "== all claims in $P"; cb "$FNS"
true
FN=printer_and_main_workspace_modes_remain_dispatchable
echo "== RED a: R2WorkspaceObjectReadMint deleted from the FleetConvergeWorkflowMode declaration"
grep -c '^  | R2WorkspaceObjectReadMint$' $W; sed -i '/^  | R2WorkspaceObjectReadMint$/d' $W; cb "$FN,observer_has_no_fleet_ssh_demand"; grep -aoE 'closure diagnostics[^)]{0,300}' /tmp/cb.out | head -2; git checkout -q -- $W
echo "== RED b: R2WorkspaceObjectReadMint removed from fleet_converge_workflow_modes only"
L=$(grep -n '^data fleet_converge_workflow_modes:' $W | cut -d: -f1); echo "line $L count $(sed -n ${L}p $W | grep -o ' R2WorkspaceObjectReadMint,' | wc -l)"
sed -i "${L}s/ R2WorkspaceObjectReadMint,//" $W; cb "$FN,observer_has_no_fleet_ssh_demand"; git checkout -q -- $W
