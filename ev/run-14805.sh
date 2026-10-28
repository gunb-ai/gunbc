set -u
ROOT=$(pwd)
git fetch -q origin c4-census-int-consumers 2>&1 | tail -2
echo "FETCHED_HEAD $(git rev-parse FETCH_HEAD) (expected 2a0b5d2185d8700091d3979709f5eb23f9a79422)"
git -C "$ROOT" worktree add -q --detach /tmp/co FETCH_HEAD && cd /tmp/co && echo "CO_HEAD $(git rev-parse HEAD)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error|could not" | head
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
F=dag/test/claim/fleet_fan_mint_seal_witness.dag
FN=witness_direct_literal_is_refused,witness_a_legal_outside_module_compiles_clean,witness_a_broken_outside_module_is_not_clean,a_census_that_cannot_run_is_unobserved_not_clean
cb(){ $B $R --entry "$1" --functions "$2" 2>&1 | grep -aE '^(PASS|FAIL|UNOBS)' | cut -c1-160; }
echo "== head"; cb $F $FN
echo "== MUTATED: unobserved arm answers false -> RED must FAIL, controls PASS"
sed -i 's/CensusUnobserved { cause: _ } => true/CensusUnobserved { cause: _ } => false/' $F
cb $F $FN
