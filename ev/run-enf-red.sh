set -u
ROOT=$(pwd)
git fetch -q origin c4-enforcement-consistency 2>&1|tail -2; HEADSHA=$(git rev-parse FETCH_HEAD)
git fetch -q origin main; BASESHA=$(git rev-parse FETCH_HEAD)
echo "HEAD $HEADSHA BASE(main) $BASESHA"
git -C "$ROOT" worktree add -q --detach /tmp/co-red $HEADSHA && cd /tmp/co-red && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
G=src/v2/test/claim/enforcement_live_witness.dag
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-200; }
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
