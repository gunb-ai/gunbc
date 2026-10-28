set -u
ROOT=$(pwd)
git fetch -q origin c4-enforcement-consistency; HEADSHA=$(git rev-parse FETCH_HEAD)
git fetch -q origin main; BASESHA=$(git rev-parse FETCH_HEAD)
echo "BASE(main) $BASESHA"
git -C "$ROOT" worktree add -q --detach /tmp/co $BASESHA && cd /tmp/co
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch
R="--wet --source-root dag --source-root src/v2"
echo "== FULL TEXT at main"
$B $R --entry src/v2/test/claim/enforcement_live_witness.dag --functions enforcement_consistency_gate_holds 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-1800
echo "== FULL TEXT: restored witness file (head)"
git checkout -q $HEADSHA -- src dag
$B $R --entry src/v2/test/claim/walk_plan_schedule_lens_test.dag --functions walk_plan_schedule_lens_synthetic_controls_hold 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-1800
