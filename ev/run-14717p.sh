set -u
git fetch -q origin main c4-boot-ref-witnesses
P=dag/test/claim/printer_workflow_binding_witness.dag; R="--wet --source-root dag --source-root src/v2"
for rev in b129ed84b691a129fdf98872967de60c2fbe6ef0 b47831b2426b9189ce222ed4c0fe29a96379cb28; do d=/tmp/co-$rev; git worktree add -q --detach $d $rev; ( cd $d
 cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error" | head -3
 echo "== REV $rev BIN $(sha256sum target/release/claim_batch | cut -c1-16)"
 target/release/claim_batch $R --entry $P --functions printer_and_main_workspace_modes_remain_dispatchable 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' ); done
