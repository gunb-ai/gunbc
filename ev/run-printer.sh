set -u; MODE=${1:-}; G=/sys/fs/cgroup/kw$$; mkdir -p $G && echo 48G > $G/memory.max && echo $$ > $G/cgroup.procs && echo "CGROUP $G $(cat $G/memory.max)"
git fetch -q origin main
R="--wet --source-root dag --source-root src/v2"; FN=printer_and_main_workspace_modes_remain_dispatchable
run(){ rev=$1; d=/tmp/cp-$rev; git worktree add -q --detach $d $rev; cd $d
 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error" | head -3
 P=$(git ls-files | grep 'test/claim/printer_workflow_binding_witness' | head -1)
 echo "== REV $(git rev-parse HEAD) BIN $(sha256sum target/release/claim_batch | cut -c1-16) FILE $P"
 target/release/claim_batch $R --entry $P --functions $FN > /tmp/cb.out 2>&1; echo "EXIT $?"; grep -aE '^(PASS|FAIL|UNOBS)|panicked' /tmp/cb.out | cut -c1-400; }
if [ "$MODE" = A1 ]; then ( run fa44b981025^1 ); exit; fi; if [ "$MODE" = A2 ]; then ( run fa44b981025 ); exit; fi
( run b129ed84b691a129fdf98872967de60c2fbe6ef0
  cat >> $P <<'X'
test fn probe_surviving_four_modes_dispatchable() -> Bool {
  all(["printer","workspace_commissioning_plan","r2_workspace_object_read_mint","r2_workspace_object_write_mint"],mode=>count(filter(fleet_converge_mode_options,m=>m==mode))==1)
}
test fn probe_one_retired_mode_absent() -> Bool {
  count(filter(fleet_converge_mode_options,m=>m=="spark_arm_group_load"))==0
}
X
  echo "== PROBES at main"; target/release/claim_batch $R --entry $P --functions probe_surviving_four_modes_dispatchable,probe_one_retired_mode_absent,$FN 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' )
