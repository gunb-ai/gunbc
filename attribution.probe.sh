set -e
E=scratch-probe/test/claim/fn_index_attribution_probe.dag
FN=a_acquisition_only,b_closure_only,c_walk_only,d_walk_plus_findings,e_acquisition_only_bigger_pool,f_walk_only_bigger_pool_same_closure
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for r in 1 2; do
  echo "===== attribution rep $r ====="
  ./target/release/claim_batch --source-root dag --source-root src/v2 --source-root scratch-probe --entry $E --functions $FN 2>&1 | grep -E "^\[witness\]|^PASS|^FAIL|^claim_batch:"
done
