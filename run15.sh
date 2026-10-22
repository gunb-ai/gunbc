set -u
HEAD_SHA=$(git rev-parse HEAD)
cp dag/test/claim/generic_arity_n_descent_witness.dag /tmp/witness.keep
run() {
  tag=$1
  cargo build --release -p v1-compiler --bin claim_batch --bin claim_executor 2>&1 | tail -1
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen.log 2>&1; echo "[$tag] REGEN_RC=$?"
  cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
  F=dag/test/claim/generic_arity_n_descent_witness.dag
  fns=$(grep -oE "^test fn [a-z0-9_]+" $F | awk '{print $3}' | paste -sd,)
  target/release/claim_batch --source-root dag --source-root src/v2 --entry $F --functions "$fns" 2>&1 | grep -E "^(PASS|FAIL|UNOBSERVED)" | sed "s/^/[$tag] /"
}
git checkout -q -f c0b7732760 && cp /tmp/witness.keep dag/test/claim/generic_arity_n_descent_witness.dag
run BASE_main_plus_REDs
git checkout -q -f $HEAD_SHA
run HEAD_readers_1_2
echo ALLDONE
