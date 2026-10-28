set -u
ROOT=$(pwd)
git fetch -q origin c4-enforcement-consistency 2>&1|tail -2; HEADSHA=$(git rev-parse FETCH_HEAD)
git fetch -q origin main; BASESHA=$(git rev-parse FETCH_HEAD)
echo "HEAD $HEADSHA BASE(main) $BASESHA"
git -C "$ROOT" worktree add -q --detach /tmp/co $HEADSHA && cd /tmp/co && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
G=src/v2/test/claim/enforcement_live_witness.dag
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-200; }
SC=src/v2/test/claim/walk_plan_schedule_lens_test.dag
echo "== HEAD: gate + control sibling + the re-homed witness"
cb $G enforcement_consistency_gate_holds,only_cost_contract_carries_red_control_probe
cb $SC walk_plan_schedule_lens_synthetic_controls_hold
echo "== IMPORTFIX-ONLY (#14887 tree = head minus my 3 changes): does the consumer cause remain?"
IF=$(git -C "$ROOT" rev-parse origin/session/merry-hawk-432-importfix); git fetch -q origin session/merry-hawk-432-importfix; IF=$(git rev-parse FETCH_HEAD)
git checkout -q $IF -- src/v2/lens/enforcement/contract.dag dag/gunbc/commit_workflow.dag; git rm -qf src/v2/test/claim/walk_plan_schedule_lens_test.dag
cb $G enforcement_consistency_gate_holds
git reset -q --hard; git checkout -q $HEADSHA -- src dag
echo "== BASE tree (origin/main)"
git checkout -q $BASESHA -- src dag; git clean -fdq src dag
cb $G enforcement_consistency_gate_holds,only_cost_contract_carries_red_control_probe
echo "== RED: head tree, binding points at a deleted file"
git checkout -q $HEADSHA -- src dag
sed -i 's#src/v2/test/claim/walk_plan_schedule_lens_test.dag",\n      function: "walk#X#' src/v2/lens/enforcement/contract.dag
python3 - <<'P'
p='src/v2/lens/enforcement/contract.dag'
s=open(p).read()
k='entry: "src/v2/test/claim/walk_plan_schedule_lens_test.dag",\n      function: "walk_plan_schedule_lens_synthetic_controls_hold"'
assert k in s
s=s.replace(k,'entry: "src/v2/test/claim/deleted_walk_plan_schedule_lens_test.dag",\n      function: "walk_plan_schedule_lens_synthetic_controls_hold"')
open(p,'w').write(s)
P
cb $G enforcement_consistency_gate_holds
